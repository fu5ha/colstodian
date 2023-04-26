use core::fmt;
use core::marker::PhantomData;
use core::ops::*;

use crate::{
    traits::*, ColAlpha, Color, Display, EncodedSrgb, LinearSrgb, Premultiplied, Separate,
};

/*
#[cfg(not(target_arch = "spirv"))]
use crate::{
    ColorResult,
    DynamicColor,
    error::DowncastError,
};
*/

use glam::{Vec4, Vec4Swizzles};
#[cfg(all(not(feature = "std"), feature = "libm"))]
use num_traits::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A strongly typed color with an alpha channel, parameterized by a color space, state, and alpha state.
///
/// See crate-level docs as well as [`ColorSpace`], [`State`] and [`AlphaState`] for more.
#[repr(transparent)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColorAlpha<Spc, St, A> {
    /// The raw values of the color. Be careful when modifying this directly.
    pub raw: Vec4,
    _pd: PhantomData<(Spc, St, A)>,
}

#[macro_export]
macro_rules! const_color_alpha {
    ($el1:expr, $el2:expr, $el3:expr, $alpha:expr) => {
        ColorAlpha {
            raw: Vec4::new($el1, $el2, $el3, $alpha),
            _pd: PhantomData,
        }
    };
}

impl<Spc, St, A> From<[f32; 4]> for ColorAlpha<Spc, St, A> {
    fn from(color: [f32; 4]) -> Self {
        Self::new(color[0], color[1], color[2], color[3])
    }
}

impl<Spc, St, A> AsRef<[f32; 4]> for ColorAlpha<Spc, St, A> {
    fn as_ref(&self) -> &[f32; 4] {
        self.raw.as_ref()
    }
}

impl<Spc, St, A> ColorAlpha<Spc, St, A> {
    /// Creates a [`ColorAlpha`] with the raw internal color elements `el1`, `el2`, `el3` and alpha value `alpha`.
    #[inline]
    pub const fn new(el1: f32, el2: f32, el3: f32, alpha: f32) -> Self {
        Self::from_raw(Vec4::new(el1, el2, el3, alpha))
    }

    /// Creates a [`ColorAlpha`] with the internal color elements all set to `el`.
    #[inline]
    pub const fn splat(el: f32) -> Self {
        Self::from_raw(Vec4::splat(el))
    }

    /// Creates a [`ColorAlpha`] with raw values contained in `raw`.
    #[inline]
    pub const fn from_raw(raw: Vec4) -> Self {
        Self {
            raw,
            _pd: PhantomData,
        }
    }

    /// Clamp the raw element values of `self` in the range [0..1]
    #[inline]
    pub fn saturate(self) -> Self {
        Self::from_raw(self.raw.min(Vec4::ONE).max(Vec4::ZERO))
    }

    /// Get the maximum element of `self`
    pub fn max_element(self) -> f32 {
        self.raw.max_element()
    }

    /// Get the minimum element of `self`
    pub fn min_element(self) -> f32 {
        self.raw.min_element()
    }

    pub const ZERO: Self = const_color_alpha!(0.0, 0.0, 0.0, 0.0);
    pub const ONE: Self = const_color_alpha!(1.0, 1.0, 1.0, 1.0);
}

/// Creates a [`ColorAlpha`] in the [`EncodedSrgb`] color space with components `r`, `g`, `b`, and `a`.
#[inline]
pub fn srgba<A: AlphaState>(r: f32, g: f32, b: f32, a: f32) -> ColorAlpha<EncodedSrgb, Display, A> {
    ColorAlpha::new(r, g, b, a)
}

/// Creates a [`ColorU8Alpha`] in the [`EncodedSrgb`] color space with components `r`, `g`, `b`, and `a`.
#[inline]
#[cfg(not(target_arch = "spirv"))]
pub fn srgba_u8<A: AlphaState>(
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> ColorU8Alpha<EncodedSrgb, Display, A> {
    ColorU8Alpha::from_raw([r, g, b, a])
}

/// Creates a [`ColorAlpha`] in the [`LinearSrgb`] color space with components `r`, `g`, `b`, and `a`
#[inline]
pub fn linear_srgba<St: State, A: AlphaState>(
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> ColorAlpha<LinearSrgb, St, A> {
    ColorAlpha::new(r, g, b, a)
}

impl<SrcSpace, St, SrcAlpha> ColorAlpha<SrcSpace, St, SrcAlpha>
where
    SrcSpace: ColorSpace,
    SrcAlpha: AlphaState,
{
    /// Converts from one color space and alpha state to another. Must be the same
    /// reference state.
    ///
    /// * If converting from [Premultiplied] to [Separate], you must ensure that `self.alpha != 0.0`, otherwise
    /// a divide by 0 will occur and `Inf`s will result.
    pub fn convert<DstSpace, DstAlpha>(self) -> ColorAlpha<DstSpace, St, DstAlpha>
    where
        DstSpace: ConvertFromRaw<SrcSpace>,
        DstAlpha: AlphaState,
    {
        let alpha = self.raw.w;

        let linear = <DstSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(self.raw.xyz());

        let separate = <SrcAlpha as ConvertToAlphaRaw<Separate>>::convert_raw(linear, alpha);

        let dst_linear = <DstSpace as ConvertFromRaw<SrcSpace>>::linear_part_raw(separate);

        let dst_alpha = <DstAlpha as ConvertFromAlphaRaw<Separate>>::convert_raw(dst_linear, alpha);

        let dst = <DstSpace as ConvertFromRaw<SrcSpace>>::dst_transform_raw(dst_alpha);

        ColorAlpha::from_raw(dst.extend(alpha))
    }

    /// Converts from one color space and state to another.
    ///
    /// This works the same as [`convert`][Color::convert] except there is only one type parameter, the
    /// "[Query][ColorAlphaConversionQuery]".
    ///
    /// The query is meant to be one of:
    /// * A [`ColorSpace`]
    /// * A [`AlphaState`]
    /// * A [`ColorAlpha`] (in which case it will be converted to that color's space and alpha state)
    ///
    /// This query is slightly more generic than the ones on [`convert`][ColorAlpha::convert], which
    /// means that the Rust type system is usually not able to infer the query without you explicitly giving one.
    ///
    /// This can be useful in conjunction with defined type aliases for predefined [`ColorAlpha`] types.
    pub fn convert_to<Query>(self) -> ColorAlpha<Query::DstSpace, St, Query::DstAlpha>
    where
        Query: ColorAlphaConversionQuery<SrcSpace, SrcAlpha>,
    {
        self.convert::<Query::DstSpace, Query::DstAlpha>()
    }

    /// Converts `self` to the provided `DstAlpha` [`AlphaState`].
    ///
    /// * If converting to the same state, this is a no-op.
    /// * If converting from [Premultiplied] to [Separate], you must ensure that `self.alpha != 0.0`, otherwise
    /// a divide by 0 will occur and `Inf`s will result.
    pub fn convert_alpha<DstAlpha: ConvertFromAlphaRaw<SrcAlpha> + AlphaState>(
        self,
    ) -> ColorAlpha<SrcSpace, St, DstAlpha> {
        let raw = self.raw.xyz();
        let alpha = self.raw.w;
        let converted = <DstAlpha as ConvertFromAlphaRaw<SrcAlpha>>::convert_raw(raw, alpha);
        ColorAlpha::from_raw(converted.extend(alpha))
    }

    /// Interprets this color as `DstSpace`. This assumes you have done an external computation/conversion such that this
    /// cast is valid.
    pub fn cast_space<DstSpace: ColorSpace>(self) -> ColorAlpha<DstSpace, St, SrcAlpha> {
        ColorAlpha::from_raw(self.raw)
    }

    /// Changes this color's alpha state. This assumes that you have done some kind of computation/conversion such that this
    /// cast is valid.
    pub fn cast_alpha_state<DstAlpha: AlphaState>(self) -> ColorAlpha<SrcSpace, St, DstAlpha> {
        ColorAlpha::from_raw(self.raw)
    }

    /// Changes this color's state. This assumes that you have done some kind of computation/conversion such that this
    /// cast is valid.
    pub fn cast_state<DstSt: State>(self) -> ColorAlpha<SrcSpace, DstSt, SrcAlpha> {
        ColorAlpha::from_raw(self.raw)
    }

    /// Changes this color's alpha state. This assumes that you have done some kind of computation/conversion such that this
    /// cast is valid.
    pub fn cast<DstSpace: ColorSpace, DstSt: State, DstAlpha: AlphaState>(
        self,
    ) -> ColorAlpha<DstSpace, DstSt, DstAlpha> {
        ColorAlpha::from_raw(self.raw)
    }
}

impl<Spc: WorkingColorSpace, St> ColorAlpha<Spc, St, Separate> {
    /// Blend `self`'s color values with the color values from `other`. Does not blend alpha.
    pub fn blend(
        self,
        other: ColorAlpha<Spc, St, Separate>,
        factor: f32,
    ) -> ColorAlpha<Spc, St, Separate> {
        ColorAlpha::from_raw(
            self.raw
                .xyz()
                .lerp(other.raw.xyz(), factor)
                .extend(self.raw.w),
        )
    }

    /// Blend `self`'s color values with the color values from `other`. Also blends alpha.
    pub fn blend_alpha(
        self,
        other: ColorAlpha<Spc, St, Separate>,
        factor: f32,
    ) -> ColorAlpha<Spc, St, Separate> {
        ColorAlpha::from_raw(self.raw.lerp(other.raw, factor))
    }
}

impl<Spc: LinearColorSpace, St: State, A: AlphaState> ColorAlpha<Spc, St, A>
where
    Premultiplied: ConvertFromAlphaRaw<A>,
{
    /// Premultiplies `self` by multiplying its color components by its alpha. Does nothing if `self` is already premultiplied.
    pub fn premultiply(self) -> ColorAlpha<Spc, St, Premultiplied> {
        let raw = self.raw.xyz();
        let alpha = self.raw.w;
        let converted = <Premultiplied as ConvertFromAlphaRaw<A>>::convert_raw(raw, alpha);
        ColorAlpha::from_raw(converted.extend(alpha))
    }
}

impl<Spc: LinearColorSpace, St, A: AlphaState> ColorAlpha<Spc, St, A>
where
    Separate: ConvertFromAlphaRaw<A>,
{
    /// Separates `self` by dividing its color components by its alpha. Does nothing if `self` is already separate.
    ///
    /// * You must ensure that `self.alpha != 0.0`, otherwise
    /// a divide by 0 will occur and `Inf`s will result.
    pub fn separate(self) -> ColorAlpha<Spc, St, Separate> {
        let raw = self.raw.xyz();
        let alpha = self.raw.w;
        let converted = <Separate as ConvertFromAlphaRaw<A>>::convert_raw(raw, alpha);
        ColorAlpha::from_raw(converted.extend(alpha))
    }
}

impl<Spc: NonlinearColorSpace, St, A: AlphaState> ColorAlpha<Spc, St, A> {
    /// Convert `self` into the closest linear color space.
    #[cfg(not(target_arch = "spirv"))]
    pub fn linearize(self) -> ColorAlpha<Spc::LinearSpace, St, A> {
        use kolor::details::{color::TransformFn, transform::ColorTransform};
        let spc = Spc::SPACE;
        ColorAlpha::from_raw(
            ColorTransform::new(spc.transform_function(), TransformFn::NONE)
                .unwrap()
                .apply(self.raw.xyz(), spc.white_point())
                .extend(self.raw.w),
        )
    }
}

impl<SrcSpace: EncodedColorSpace, St, A: AlphaState> ColorAlpha<SrcSpace, St, A> {
    /// Decode `self` into its decoded ([working][WorkingColorSpace]) color space,
    /// which allows many more operations to be performed.
    pub fn decode(self) -> ColorAlpha<SrcSpace::DecodedSpace, St, A> {
        let raw_xyz =
            <SrcSpace::DecodedSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(self.raw.xyz());
        ColorAlpha::from_raw(raw_xyz.extend(self.raw.w))
    }
}

impl<Spc, St, A> From<ColorAlpha<Spc, St, A>> for Color<Spc, St>
where
    Spc: ColorSpace,
    St: State,
    A: AlphaState,
    Premultiplied: ConvertFromAlphaRaw<A>,
{
    fn from(c: ColorAlpha<Spc, St, A>) -> Self {
        c.into_color()
    }
}

impl<Spc, St, A> ColorAlpha<Spc, St, A> {
    /// Converts `self` to a [`Color`] by stripping off the alpha component.
    pub fn into_color_no_premultiply(self) -> Color<Spc, St> {
        Color::from_raw(self.raw.xyz())
    }
}

impl<Spc, St, A> ColorAlpha<Spc, St, A>
where
    Spc: ColorSpace,
    A: AlphaState,
    Premultiplied: ConvertFromAlphaRaw<A>,
{
    /// Converts `self` to a [`Color`] by first premultiplying `self` (if premultiplying makes sense for the current color space)
    /// and then stripping off the alpha component.
    pub fn into_color(self) -> Color<Spc, St> {
        if Spc::SPACE != Spc::LinearSpace::SPACE {
            Color::from_raw(self.convert_alpha::<Premultiplied>().raw.xyz())
        } else {
            Color::from_raw(self.raw.xyz())
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<Spc: AsU8, St, A: AlphaState> ColorAlpha<Spc, St, A> {
    /// Convert `self` to a [`ColorU8Alpha`] of identical type.
    /// All components of `self` will be clamped to be in range `[0..1]`.
    pub fn to_u8(self) -> ColorU8Alpha<Spc, St, A> {
        fn f32_to_u8(x: f32) -> u8 {
            (x * 255.0).min(255.0).max(0.0).round() as u8
        }
        ColorU8Alpha::from_raw([
            f32_to_u8(self.raw.x),
            f32_to_u8(self.raw.y),
            f32_to_u8(self.raw.z),
            f32_to_u8(self.raw.w),
        ])
    }

    /// Decode a [`ColorU8Alpha`] into a [`ColorAlpha`] of identical type.
    pub fn from_u8(encoded: ColorU8Alpha<Spc, St, A>) -> ColorAlpha<Spc, St, A> {
        fn u8_to_f32(x: u8) -> f32 {
            x as f32 / 255.0
        }
        ColorAlpha::new(
            u8_to_f32(encoded[0]),
            u8_to_f32(encoded[1]),
            u8_to_f32(encoded[2]),
            u8_to_f32(encoded[3]),
        )
    }
}

impl<SrcSpace, DstSpace, St, SrcAlpha, DstAlpha> ColorInto<ColorAlpha<DstSpace, St, DstAlpha>>
    for ColorAlpha<SrcSpace, St, SrcAlpha>
where
    DstSpace: ConvertFromRaw<SrcSpace>,
    SrcSpace: ColorSpace,
    DstAlpha: ConvertFromAlphaRaw<SrcAlpha> + AlphaState,
    SrcAlpha: AlphaState,
{
    fn into(self) -> ColorAlpha<DstSpace, St, DstAlpha> {
        self.convert()
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<Spc, St, A> fmt::Display for ColorAlpha<Spc, St, A>
where
    Spc: ColorSpace + fmt::Display,
    Spc::ComponentStruct: fmt::Display,
    St: State + fmt::Display,
    A: AlphaState + fmt::Display,
    ColorAlpha<Spc, St, A>: Deref<Target = ColAlpha<Spc::ComponentStruct>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ColorAlpha<{}, {}, {}>: ({})",
            Spc::default(),
            St::default(),
            A::default(),
            self.deref()
        )
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<Spc, St, A> fmt::Debug for ColorAlpha<Spc, St, A>
where
    Spc: ColorSpace + fmt::Display,
    Spc::ComponentStruct: fmt::Display,
    St: State + fmt::Display,
    A: AlphaState + fmt::Display,
    ColorAlpha<Spc, St, A>: Deref<Target = ColAlpha<Spc::ComponentStruct>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl<Spc, St, A> Copy for ColorAlpha<Spc, St, A> {}

impl<Spc, St, A> Clone for ColorAlpha<Spc, St, A> {
    fn clone(&self) -> ColorAlpha<Spc, St, A> {
        *self
    }
}

impl<Spc, St, A> PartialEq for ColorAlpha<Spc, St, A> {
    fn eq(&self, other: &ColorAlpha<Spc, St, A>) -> bool {
        self.raw == other.raw
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl<Spc, St, A> bytemuck::Zeroable for ColorAlpha<Spc, St, A> {}
#[cfg(feature = "bytemuck")]
unsafe impl<Spc, St, A> bytemuck::TransparentWrapper<Vec4> for ColorAlpha<Spc, St, A> {}
#[cfg(feature = "bytemuck")]
unsafe impl<Spc: 'static, St: 'static, A: 'static> bytemuck::Pod for ColorAlpha<Spc, St, A> {}

macro_rules! impl_op_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, St, A> $op for ColorAlpha<Spc, St, A> {
            type Output = ColorAlpha<Spc, St, A>;
            fn $op_func(self, rhs: ColorAlpha<Spc, St, A>) -> Self::Output {
                ColorAlpha::from_raw(self.raw.$op_func(rhs.raw))
            }
        }

        impl<Spc: LinearColorSpace, St, A> $op<Vec4> for ColorAlpha<Spc, St, A> {
            type Output = ColorAlpha<Spc, St, A>;
            fn $op_func(self, rhs: Vec4) -> Self::Output {
                ColorAlpha::from_raw(self.raw.$op_func(rhs))
            }
        }
    };
}

macro_rules! impl_binop_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, St, A> $op for ColorAlpha<Spc, St, A> {
            fn $op_func(&mut self, rhs: ColorAlpha<Spc, St, A>) {
                self.raw.$op_func(rhs.raw)
            }
        }

        impl<Spc: LinearColorSpace, St, A> $op<Vec4> for ColorAlpha<Spc, St, A> {
            fn $op_func(&mut self, rhs: Vec4) {
                self.raw.$op_func(rhs)
            }
        }
    };
}

macro_rules! impl_op_color_float {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, St, A> $op<f32> for ColorAlpha<Spc, St, A> {
            type Output = ColorAlpha<Spc, St, A>;
            fn $op_func(self, rhs: f32) -> Self::Output {
                ColorAlpha::from_raw(self.raw.$op_func(rhs))
            }
        }

        impl<Spc: LinearColorSpace, St, A> $op<ColorAlpha<Spc, St, A>> for f32 {
            type Output = ColorAlpha<Spc, St, A>;
            fn $op_func(self, rhs: ColorAlpha<Spc, St, A>) -> Self::Output {
                ColorAlpha::from_raw(self.$op_func(rhs.raw))
            }
        }
    };
}

impl_op_color!(Add, add);
impl_op_color!(Sub, sub);
impl_op_color!(Mul, mul);
impl_op_color!(Div, div);

impl_binop_color!(AddAssign, add_assign);
impl_binop_color!(SubAssign, sub_assign);
impl_binop_color!(MulAssign, mul_assign);
impl_binop_color!(DivAssign, div_assign);

impl_op_color_float!(Mul, mul);
impl_op_color_float!(Div, div);

#[cfg(not(target_arch = "spirv"))]
mod color_u8 {
    use super::*;

    /// An encoded color with alpha, 8-bit per component, 32-bit total.
    ///
    /// This should only be used when space is an issue, i.e. when compressing data.
    /// Otherwise prefer a [`ColorAlpha`].
    #[derive(Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    #[repr(C)]
    pub struct ColorU8Alpha<Spc, St, A> {
        pub raw: [u8; 4],
        _pd: PhantomData<(Spc, St, A)>,
    }

    impl<Spc, St, A> Clone for ColorU8Alpha<Spc, St, A> {
        fn clone(&self) -> ColorU8Alpha<Spc, St, A> {
            *self
        }
    }

    impl<Spc, St, A> Copy for ColorU8Alpha<Spc, St, A> {}

    #[cfg(feature = "bytemuck")]
    unsafe impl<Spc, St, A> bytemuck::Zeroable for ColorU8Alpha<Spc, St, A> {}
    #[cfg(feature = "bytemuck")]
    unsafe impl<Spc, St, A> bytemuck::TransparentWrapper<[u8; 4]> for ColorU8Alpha<Spc, St, A> {}
    #[cfg(feature = "bytemuck")]
    unsafe impl<Spc: 'static, St: 'static, A: 'static> bytemuck::Pod for ColorU8Alpha<Spc, St, A> {}

    impl<Spc, St, A> Index<usize> for ColorU8Alpha<Spc, St, A> {
        type Output = u8;

        fn index(&self, i: usize) -> &u8 {
            &self.raw[i]
        }
    }

    impl<Spc, St, A> IndexMut<usize> for ColorU8Alpha<Spc, St, A> {
        fn index_mut(&mut self, i: usize) -> &mut u8 {
            &mut self.raw[i]
        }
    }

    impl<Spc, St, A> AsRef<[u8; 4]> for ColorU8Alpha<Spc, St, A> {
        fn as_ref(&self) -> &[u8; 4] {
            &self.raw
        }
    }

    impl<Spc, St, A> ColorU8Alpha<Spc, St, A> {
        pub fn new(x: u8, y: u8, z: u8, w: u8) -> Self {
            Self {
                raw: [x, y, z, w],
                _pd: PhantomData,
            }
        }

        pub fn from_raw(raw: [u8; 4]) -> Self {
            Self {
                raw,
                _pd: PhantomData,
            }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    impl<Spc: AsU8, St, A> ColorU8Alpha<Spc, St, A> {
        /// Convert `self` to a [`ColorU8Alpha`] of identical type.
        /// All components of `self` will be clamped to be in range `[0..1]`.
        pub fn from_f32(col: ColorAlpha<Spc, St, A>) -> ColorU8Alpha<Spc, St, A> {
            fn f32_to_u8(x: f32) -> u8 {
                (x * 255.0).min(255.0).max(0.0).round() as u8
            }
            ColorU8Alpha::from_raw([
                f32_to_u8(col.raw.x),
                f32_to_u8(col.raw.y),
                f32_to_u8(col.raw.z),
                f32_to_u8(col.raw.w),
            ])
        }

        /// Decode a [`ColorU8Alpha`] into a [`ColorAlpha`] of identical type.
        pub fn to_f32(self) -> ColorAlpha<Spc, St, A> {
            fn u8_to_f32(x: u8) -> f32 {
                x as f32 / 255.0
            }
            ColorAlpha::new(
                u8_to_f32(self[0]),
                u8_to_f32(self[1]),
                u8_to_f32(self[2]),
                u8_to_f32(self[3]),
            )
        }
    }

    impl<Spc, St, A> From<ColorU8Alpha<Spc, St, A>> for u32 {
        fn from(c: ColorU8Alpha<Spc, St, A>) -> u32 {
            (u32::from(c.raw[0]) << 24)
                | (u32::from(c.raw[1]) << 16)
                | (u32::from(c.raw[2]) << 8)
                | u32::from(c.raw[3])
        }
    }

    impl<Spc, St, A> From<u32> for ColorU8Alpha<Spc, St, A> {
        fn from(c: u32) -> Self {
            Self::new((c >> 24) as u8, (c >> 16) as u8, (c >> 8) as u8, c as u8)
        }
    }

    impl<Spc, St, A> From<[u8; 4]> for ColorU8Alpha<Spc, St, A> {
        fn from(raw: [u8; 4]) -> Self {
            Self::from_raw(raw)
        }
    }

    impl<Spc, St, A> From<ColorU8Alpha<Spc, St, A>> for [u8; 4] {
        fn from(c: ColorU8Alpha<Spc, St, A>) -> Self {
            c.raw
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
pub use color_u8::ColorU8Alpha;

/*
/// A dynamic color with an alpha channel, with its space and alpha defined
/// as data. This is mostly useful for (de)serialization.
///
/// See [`ColorAlpha`], [`ColorSpace`] and [`AlphaState`] for more.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg(not(target_arch = "spirv"))]
pub struct DynamicColorAlpha {
    /// The raw tristimulus value of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec4,
    pub space: DynamicColorSpace,
    pub alpha_state: DynamicAlphaState,
}

#[cfg(not(target_arch = "spirv"))]
impl DynamicColorAlpha {
    /// Create a new [`DynamicColorAlpha`] with specified raw color components, color space, and alpha state.
    pub fn new(raw: Vec4, space: DynamicColorSpace, alpha_state: DynamicAlphaState) -> Self {
        Self {
            raw,
            space,
            alpha_state,
        }
    }

    /// Converts `self` to a [`DynamicColor`] by first premultiplying `self` if it is not already
    /// and then stripping off the alpha component.
    pub fn into_color(self) -> DynamicColor {
        let color_alpha = self.convert_alpha_state(DynamicAlphaState::Premultiplied);
        DynamicColor::new(color_alpha.raw.xyz(), self.space, DynamicState::Display)
    }

    /// Converts `self` to a [`DynamicColor`] by stripping off the alpha component, without checking
    /// whether it is premultiplied or not.
    pub fn into_color_no_premultiply(self) -> DynamicColor {
        DynamicColor::new(self.raw.xyz(), self.space, DynamicState::Display)
    }

    /// Converts from one color space and state to another.
    ///
    /// * If converting from [Premultiplied][DynamicAlphaState::Premultiplied] to [Separate][DynamicAlphaState::Separate], if
    /// `self`'s alpha is 0.0, the resulting color values will not be changed.
    pub fn convert(mut self, dst_space: DynamicColorSpace, dst_alpha: DynamicAlphaState) -> Self {
        let conversion = kolor::ColorConversion::new(self.space, dst_space);

        // linearize
        self.raw = conversion.apply_src_transform(self.raw.xyz()).extend(1.0);

        // separate
        self = self.convert_alpha_state(DynamicAlphaState::Separate);

        // linear color conversion
        self.raw = conversion.apply_linear_part(self.raw.xyz()).extend(1.0);

        // convert to dst alpha state
        self = self.convert_alpha_state(dst_alpha);

        // dst transform
        self.raw = conversion.apply_dst_transform(self.raw.xyz()).extend(1.0);
        self.space = dst_space;

        self
    }

    /// Convert `self` to the specified space and downcast it to a typed [`ColorAlpha`] with the space
    /// and state specified.
    pub fn downcast_convert<DstSpace, DstAlpha>(self) -> ColorAlpha<DstSpace, DstAlpha>
    where
        DstSpace: ColorSpace,
        DstAlpha: AlphaState,
    {
        let dst = self.convert(DstSpace::SPACE, DstAlpha::STATE);
        ColorAlpha::from_raw(dst.raw)
    }

    /// Converts `self` to the provided `dst_alpha` [`DynamicAlphaState`].
    ///
    /// * If converting to the same state, this is a no-op.
    /// * If converting from [Premultiplied][DynamicAlphaState::Premultiplied] to [Separate][DynamicAlphaState::Separate], if
    /// `self`'s alpha is 0.0, the resulting color values will not be changed.
    pub fn convert_alpha_state(self, dst_alpha: DynamicAlphaState) -> DynamicColorAlpha {
        let col = match (self.alpha_state, dst_alpha) {
            (DynamicAlphaState::Separate, DynamicAlphaState::Premultiplied) => {
                self.raw.xyz() * self.raw.w
            }
            (DynamicAlphaState::Premultiplied, DynamicAlphaState::Separate) => {
                if self.raw.w != 0.0 {
                    self.raw.xyz() / self.raw.w
                } else {
                    self.raw.xyz()
                }
            }
            _ => self.raw.xyz(),
        };

        Self {
            raw: col.extend(self.raw.w),
            space: self.space,
            alpha_state: dst_alpha,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<'a> From<&'a dyn AnyColorAlpha> for DynamicColorAlpha {
    fn from(color: &'a dyn AnyColorAlpha) -> DynamicColorAlpha {
        color.dynamic()
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<C: AnyColorAlpha> DynColorAlpha for C {
    /// Attempt to convert to a typed [`ColorAlpha`]. Returns an error if `self`'s color space and alpha state do not match
    /// the given types.
    fn downcast<Spc: ColorSpace, A: AlphaState>(&self) -> ColorResult<ColorAlpha<Spc, A>> {
        if self.space() != Spc::SPACE {
            return Err(DowncastError::MismatchedSpace(self.space(), Spc::SPACE).into());
        }

        if self.alpha_state() != A::STATE {
            return Err(DowncastError::MismatchedAlphaState(self.alpha_state(), A::STATE).into());
        }

        Ok(ColorAlpha::from_raw(self.raw()))
    }

    /// Convert to a typed `ColorAlpha` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    fn downcast_unchecked<Spc: ColorSpace, A: AlphaState>(&self) -> ColorAlpha<Spc, A> {
        ColorAlpha::from_raw(self.raw())
    }
}
*/
