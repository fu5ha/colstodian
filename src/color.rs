use crate::traits::*;
use crate::{
    error::{DowncastError, DynamicConversionError},
    tonemapper::Tonemapper,
    AcesCg, ColorResult, Display, DynamicAlphaState, DynamicColorSpace, DynamicState, EncodedSrgb,
    LinearSrgb, Scene, Separate,
};

use glam::Vec3;
#[cfg(all(not(feature = "std"), feature = "libm"))]
use num_traits::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt;
use core::marker::PhantomData;
use core::ops::*;

mod color_alpha;
pub use color_alpha::{linear_srgba, srgba, srgba_u8, ColorAlpha, DynamicColorAlpha};

/// A strongly typed color, parameterized by a color space and state.
///
/// See crate-level docs as well as [`ColorSpace`] and [`State`] for more.
#[repr(transparent)]
pub struct Color<Spc, St> {
    /// The raw values of the color. Be careful when modifying this directly.
    pub raw: Vec3,
    _pd: PhantomData<(Spc, St)>,
}

impl<Spc, St> Color<Spc, St> {
    /// Creates a [`Color`] with the internal color elements `el1`, `el2`, `el3`.
    #[inline]
    pub fn new(el1: f32, el2: f32, el3: f32) -> Self {
        Self::from_raw(Vec3::new(el1, el2, el3))
    }

    /// Creates a [`Color`] with raw values contained in `raw`.
    #[inline]
    pub const fn from_raw(raw: Vec3) -> Self {
        Self {
            raw,
            _pd: PhantomData,
        }
    }

    /// Get the maximum element of `self`
    pub fn max_element(self) -> f32 {
        self.raw.max_element()
    }

    /// Get the minimum element of `self`
    pub fn min_element(self) -> f32 {
        self.raw.min_element()
    }
}

impl<Spc: WorkingColorSpace, St> Color<Spc, St> {
    /// Clamp the raw element values of `self` in the range [0..1]
    #[inline]
    pub fn saturate(self) -> Self {
        Self::from_raw(self.raw.min(Vec3::ONE).max(Vec3::ZERO))
    }
}

/// Creates a [`Color`] in the [`EncodedSrgb`] color space with components `r`, `g`, and `b`.
#[inline]
pub fn srgb(r: f32, g: f32, b: f32) -> Color<EncodedSrgb, Display> {
    Color::new(r, g, b)
}

/// Creates a [`Color`] in the [`EncodedSrgb`] color space with components `r`, `g`, and `b`.
#[inline]
pub fn srgb_u8(r: u8, g: u8, b: u8) -> Color<EncodedSrgb, Display> {
    Color::from_u8([r, g, b])
}

/// Creates a [`Color`] in the [`LinearSrgb`] color space in the `St` [State] with components `r`, `g`, and `b`.
#[inline]
pub fn linear_srgb<St: State>(r: f32, g: f32, b: f32) -> Color<LinearSrgb, St> {
    Color::new(r, g, b)
}

/// Creates a [`Color`] in the [`AcesCg`] color space in the `St` [State] with components `r`, `g`, and `b`.
#[inline]
pub fn acescg<St: State>(r: f32, g: f32, b: f32) -> Color<AcesCg, St> {
    Color::new(r, g, b)
}

impl<SrcSpace: ColorSpace, St: State> Color<SrcSpace, St> {
    /// Converts `self` from one color space to another while retaining the same [`State`].
    ///
    /// Be careful when converting between nonlinear color spaces while in [Scene] state
    /// as the nonlinear transform functions for some color spaces are only defined within
    /// a specific dynamic range.
    pub fn convert<DstSpace: ConvertFromRaw<SrcSpace>>(self) -> Color<DstSpace, St> {
        let mut raw = self.raw;
        raw = <DstSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(raw);
        raw = <DstSpace as ConvertFromRaw<SrcSpace>>::linear_part_raw(raw);
        raw = <DstSpace as ConvertFromRaw<SrcSpace>>::dst_transform_raw(raw);
        Color::from_raw(raw)
    }

    /// Converts `self` from one color space to another while retaining the same [`State`].
    ///
    /// This works the same as [`convert`][Color::convert] except there is only one type parameter, the
    /// "[Query][ColorConversionQuery]".
    ///
    /// The query is meant to be one of:
    /// * A [`ColorSpace`]
    /// * A [`Color`] (in which case it will be converted to that color's space).
    ///
    /// This query is slightly more generic than the ones on [`convert`][ColorAlpha::convert], which
    /// means that the Rust type system is usually not able to infer the query without you explicitly giving one.
    ///
    /// This can be useful in conjunction with defined type aliases for predefined [`Color`] types.
    pub fn convert_to<Query>(self) -> Color<Query::DstSpace, St>
    where
        Query: ColorConversionQuery<SrcSpace, St>,
    {
        self.convert::<Query::DstSpace>()
    }

    /// Interprets this color as `DstSpace`. This assumes you have done an external computation/conversion such that this
    /// cast is valid.
    pub fn cast_space<DstSpace: ColorSpace>(self) -> Color<DstSpace, St> {
        Color::from_raw(self.raw)
    }

    /// Interprets this color as `DstSpace` and `DstState`. This assumes you have done an external computation/conversion such that this
    /// cast is valid.
    pub fn cast<DstSpace: ColorSpace, DstSt: State>(self) -> Color<DstSpace, DstSt> {
        Color::from_raw(self.raw)
    }
}

impl<SrcSpace: EncodedColorSpace, St: State> Color<SrcSpace, St> {
    /// Decode `self` into its decoded ([working][WorkingColorSpace]) color space,
    /// which allows many more operations to be performed.
    pub fn decode(self) -> Color<SrcSpace::DecodedSpace, St> {
        let mut raw = self.raw;
        raw = <SrcSpace::DecodedSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(raw);
        Color::from_raw(raw)
    }
}

impl<SrcSpace: ColorSpace, St: State> Color<SrcSpace, St> {
    /// Convert `self` into the closest linear color space.
    ///
    /// If `self` is already in a linear color space, this is a no-op.
    pub fn linearize(self) -> Color<SrcSpace::LinearSpace, St> {
        let mut raw = self.raw;
        raw = <SrcSpace::LinearSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(raw);
        Color::from_raw(raw)
    }
}

impl<Spc: LinearColorSpace, SrcSt> Color<Spc, SrcSt> {
    /// Converts this color from one state to another.
    ///
    /// This conversion is usecase and even instance dependent.
    /// For example, converting a material's emissive texture value, a [`Display`]-referred color, to a [`Scene`]-referred
    /// color might take the form of a multiplication which scales the power of said emission into [`Scene`]-referred irradiance. On the other hand,
    /// converting a final [`Scene`]-referred color to a [`Display`]-referred color should be done with some kind of tonemapping
    /// operator. For a built-in, configurable tonemapper, see [`Tonemapper`].
    ///
    /// Note that the conversion function gives a raw color value, as the state of the color during the intermediate steps of the conversion
    /// is not really well defined. Therefore it's easier to just work on the raw values without type safety.
    pub fn convert_state<DstSt, F>(self, conversion_function: F) -> Color<Spc, DstSt>
    where
        F: FnOnce(Vec3) -> Vec3,
    {
        Color::from_raw(conversion_function(self.raw))
    }

    /// Changes this color's State. This assumes that you have done some kind of conversion externally,
    /// or that the proper conversion is simply a noop.
    pub fn cast_state<DstSt>(self) -> Color<Spc, DstSt> {
        Color::from_raw(self.raw)
    }
}

impl<Spc: LinearColorSpace> Color<Spc, Scene> {
    /// Tonemap `self` using the `tonemapper`, converting `self` from being
    /// scene-referred to being display-referred.
    pub fn tonemap(self, tonemapper: impl Tonemapper) -> Color<Spc, Display> {
        Color::from_raw(tonemapper.tonemap_raw(self.raw))
    }
}

impl<Spc: ColorSpace> Color<Spc, Display> {
    /// Converts `self` to a [`ColorAlpha`] with [`Separate`] alpha state by adding a component. This is probably what you want.
    pub fn with_alpha(self, alpha: f32) -> ColorAlpha<Spc, Separate> {
        ColorAlpha::from_raw(self.raw.extend(alpha))
    }

    /// Converts `self` to a [`ColorAlpha`] with specified [`AlphaState`] by adding an alpha component. Make sure you choose the
    /// correct alpha state! If you're not sure, you probably want [`Color::with_alpha`].
    pub fn with_alpha_state<A: AlphaState>(self, alpha: f32) -> ColorAlpha<Spc, A> {
        ColorAlpha::from_raw(self.raw.extend(alpha))
    }
}

impl<Spc: AsU8Array> Color<Spc, Display> {
    /// Convert `self` to a `[u8; 3]`. All components of `self` will be clamped to range `[0..1]`.
    pub fn to_u8(self) -> [u8; 3] {
        fn f32_to_u8(x: f32) -> u8 {
            (x * 255.0).round() as u8
        }
        [
            f32_to_u8(self.raw.x),
            f32_to_u8(self.raw.y),
            f32_to_u8(self.raw.z),
        ]
    }

    /// Decode a `[u8; 4]` into a `Color` with specified space and state.
    pub fn from_u8(encoded: [u8; 3]) -> Color<Spc, Display> {
        fn u8_to_f32(x: u8) -> f32 {
            x as f32 / 255.0
        }
        Color::new(
            u8_to_f32(encoded[0]),
            u8_to_f32(encoded[1]),
            u8_to_f32(encoded[2]),
        )
    }
}

impl<SrcSpace, DstSpace, St> ConvertTo<Color<DstSpace, St>> for Color<SrcSpace, St>
where
    DstSpace: ConvertFromRaw<SrcSpace>,
    SrcSpace: ColorSpace,
    St: State,
{
    fn convert(self) -> Color<DstSpace, St> {
        self.convert()
    }
}

impl<Spc: ColorSpace, St: State> From<Color<Spc, St>> for DynamicColor {
    fn from(color: Color<Spc, St>) -> DynamicColor {
        color.dynamic()
    }
}

impl<Spc: ColorSpace, St: State> From<Color<Spc, St>> for kolor::Color {
    fn from(color: Color<Spc, St>) -> kolor::Color {
        kolor::Color {
            value: color.raw,
            space: Spc::SPACE,
        }
    }
}

impl<Spc, St> Copy for Color<Spc, St> {}

impl<Spc, St> Clone for Color<Spc, St> {
    fn clone(&self) -> Color<Spc, St> {
        *self
    }
}

impl<Spc, St> PartialEq for Color<Spc, St> {
    fn eq(&self, other: &Color<Spc, St>) -> bool {
        self.raw == other.raw
    }
}

unsafe impl<Spc, St> bytemuck::Zeroable for Color<Spc, St> {}
unsafe impl<Spc, St> bytemuck::TransparentWrapper<Vec3> for Color<Spc, St> {}
unsafe impl<Spc: 'static, St: 'static> bytemuck::Pod for Color<Spc, St> {}

impl<Spc, St> fmt::Display for Color<Spc, St>
where
    Spc: ColorSpace,
    St: State,
    Color<Spc, St>: Deref<Target = Spc::ComponentStruct>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color<{}, {}>: ({})",
            Spc::default(),
            St::default(),
            self.deref()
        )
    }
}

impl<Spc, St> fmt::Debug for Color<Spc, St>
where
    Spc: ColorSpace,
    St: State,
    Color<Spc, St>: Deref<Target = Spc::ComponentStruct>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl<Spc: ColorSpace, St: State> AnyColor for Color<Spc, St> {
    #[inline]
    fn space(&self) -> DynamicColorSpace {
        Spc::SPACE
    }

    #[inline]
    fn state(&self) -> DynamicState {
        St::STATE
    }

    #[inline]
    fn raw(&self) -> Vec3 {
        self.raw
    }
}

macro_rules! impl_op_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc: WorkingColorSpace, St> $op for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from_raw(self.raw.$op_func(rhs.raw))
            }
        }
    };
}

macro_rules! impl_op_color_float {
    ($op:ident, $op_func:ident) => {
        impl<Spc: WorkingColorSpace, St> $op<f32> for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: f32) -> Self::Output {
                Color::from_raw(self.raw.$op_func(rhs))
            }
        }

        impl<Spc: WorkingColorSpace, St> $op<Color<Spc, St>> for f32 {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from_raw(self.$op_func(rhs.raw))
            }
        }
    };
}

impl_op_color!(Add, add);
impl_op_color!(Sub, sub);
impl_op_color!(Mul, mul);
impl_op_color!(Div, div);

impl_op_color_float!(Mul, mul);
impl_op_color_float!(Div, div);

/// A dynamic color, with its Space and State defined
/// as data. This is mostly useful for (de)serialization.
///
/// See [`Color`], [`ColorSpace`] and [`State`] for more.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct DynamicColor {
    /// The raw tristimulus value of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec3,
    pub space: DynamicColorSpace,
    pub state: DynamicState,
}

impl AnyColor for DynamicColor {
    #[inline]
    fn space(&self) -> DynamicColorSpace {
        self.space
    }

    #[inline]
    fn state(&self) -> DynamicState {
        self.state
    }

    #[inline]
    fn raw(&self) -> Vec3 {
        self.raw
    }
}

impl DynamicColor {
    /// Create a new [`DynamicColor`] with specified raw color components, color space, and state.
    pub fn new(raw: Vec3, space: DynamicColorSpace, state: DynamicState) -> Self {
        Self { raw, space, state }
    }

    /// Convert `self` to the given color space. Must not attempt to convert to or from
    /// a nonlinear color space while in scene-referred state.
    pub fn convert(self, dest_space: DynamicColorSpace) -> ColorResult<Self> {
        if self.state == DynamicState::Scene && (!self.space.is_linear() || !dest_space.is_linear())
        {
            return Err(DynamicConversionError::NonlinearConversionInSceneState(
                self.space, dest_space,
            )
            .into());
        }
        let conversion = kolor::ColorConversion::new(self.space, dest_space);
        let raw = conversion.convert(self.raw);
        Ok(Self {
            raw,
            space: dest_space,
            state: self.state,
        })
    }

    /// Convert `self`'s state to the given state using the given conversion function.
    ///
    /// `self.space` must be linear. See docs for [`Color::<Space, State>::convert_state`]
    pub fn convert_state<F>(self, dest_state: DynamicState, conversion: F) -> ColorResult<Self>
    where
        F: FnOnce(Vec3) -> Vec3,
    {
        if !self.space.is_linear() {
            return Err(DynamicConversionError::StateChangeInNonlinearSpace(
                self.space, self.state, dest_state,
            )
            .into());
        }
        Ok(Self {
            raw: conversion(self.raw),
            space: self.space,
            state: dest_state,
        })
    }

    /// Convert `self` into the closest linear color space, if it is not linear already
    pub fn linearize(self) -> Self {
        use kolor::details::{color::TransformFn, transform::ColorTransform};
        let spc = self.space;
        let raw = if let Some(transform) =
            ColorTransform::new(spc.transform_function(), TransformFn::NONE)
        {
            transform.apply(self.raw, spc.white_point())
        } else {
            self.raw
        };
        Self {
            raw,
            space: spc.as_linear(),
            state: self.state,
        }
    }

    /// Tonemap `self` using the [`Tonemapper`] `tonemapper`, converting `self` from being
    /// scene-referred to being display-referred.
    pub fn tonemap(mut self, tonemapper: impl Tonemapper) -> ColorResult<Self> {
        if self.state != DynamicState::Scene {
            return Err(DynamicConversionError::TonemapInDisplayState.into());
        }
        if !self.space.is_linear() {
            return Err(DynamicConversionError::StateChangeInNonlinearSpace(
                self.space,
                self.state,
                DynamicState::Display,
            )
            .into());
        }
        self.raw = tonemapper.tonemap_raw(self.raw);
        Ok(self)
    }

    /// Converts `self` to a [`DynamicColorAlpha`] with specified [`DynamicAlphaState`] by adding an alpha component.
    pub fn with_alpha(&self, alpha: f32, alpha_state: DynamicAlphaState) -> DynamicColorAlpha {
        DynamicColorAlpha {
            raw: self.raw.extend(alpha),
            space: self.space,
            alpha_state,
        }
    }

    pub fn from_kolor(color: kolor::Color, state: DynamicState) -> Self {
        Self::new(color.value, color.space, state)
    }
}

impl From<DynamicColor> for kolor::Color {
    fn from(color: DynamicColor) -> kolor::Color {
        kolor::Color {
            value: color.raw,
            space: color.space,
        }
    }
}

impl<C: AnyColor> DynColor for C {
    /// Attempt to convert to a typed `Color`. Returns an error if `self`'s color space and state do not match
    /// the given types.
    fn downcast<Spc: ColorSpace, St: State>(&self) -> ColorResult<Color<Spc, St>> {
        if self.space() != Spc::SPACE {
            return Err(DowncastError::MismatchedSpace(self.space(), Spc::SPACE).into());
        }

        if self.state() != St::STATE {
            return Err(DowncastError::MismatchedState(self.state(), St::STATE).into());
        }

        Ok(Color::from_raw(self.raw()))
    }

    /// Convert to a typed `Color` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    fn downcast_unchecked<Spc: ColorSpace, St: State>(&self) -> Color<Spc, St> {
        Color::from_raw(self.raw())
    }
}
