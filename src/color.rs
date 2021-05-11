use super::*;

/// A strongly typed color, parameterized by a color space and state.
///
/// See crate-level docs as well as [`ColorSpace`] and [`State`] for more.
#[derive(Derivative)]
#[derivative(Clone, Copy, PartialEq, Debug)]
pub struct Color<Spc, St> {
    /// The raw values of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec3,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Debug = "ignore")]
    _pd: PhantomData<(Spc, St)>,
}

impl<Spc, St> Color<Spc, St> {
    /// Creates a [`Color`] with the internal color elements `el1`, `el2`, `el3`.
    #[inline]
    pub fn new(el1: f32, el2: f32, el3: f32) -> Self {
        Self::from(Vec3::new(el1, el2, el3))
    }

    /// Creates a [`Color`] with raw values contained in `raw`.
    #[inline]
    pub const fn from(raw: Vec3) -> Self {
        Self {
            raw,
            _pd: PhantomData,
        }
    }

    /// Clamp the raw element values of `self` in the range [0..1]
    #[inline]
    pub fn saturate(self) -> Self {
        Self::from(self.raw.min(Vec3::ONE).max(Vec3::ZERO))
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

/// Creates a [`Color`] in the [`EncodedSrgb`] color space with components `r`, `g`, and `b`.
#[inline]
pub fn srgb(r: f32, g: f32, b: f32) -> Color<EncodedSrgb, Display> {
    Color::new(r, g, b)
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

macro_rules! impl_op_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, St> $op for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from(self.raw.$op_func(rhs.raw))
            }
        }
    };
}

macro_rules! impl_op_color_float {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, St> $op<f32> for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: f32) -> Self::Output {
                Color::from(self.raw.$op_func(rhs))
            }
        }

        impl<Spc: LinearColorSpace, St> $op<Color<Spc, St>> for f32 {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from(self.$op_func(rhs.raw))
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

impl<SrcSpace: ColorSpace, Display> Color<SrcSpace, Display> {
    /// Converts from one color space to another. This is only implemented in the generic case (for any ColorSpace)
    /// for Display-referred colors because non-linear color space transformations are often undefined for values
    /// outside the range [0..1].
    pub fn convert<DstSpace: ColorSpace>(self) -> Color<DstSpace, Display> {
        let conversion = kolor::ColorConversion::new(SrcSpace::SPACE, DstSpace::SPACE);
        Color::from(conversion.convert(self.raw))
    }

    /// Interprets this color as `DstSpace`. This assumes you have done an external computation/conversion such that this
    /// cast is valid.
    pub fn cast_space<DstSpace: ColorSpace>(self) -> Color<DstSpace, Display> {
        Color::from(self.raw)
    }
}

impl<SrcSpace, St> Color<SrcSpace, St> {
    /// Converts from a linear color space to another linear color space. This transformation ultimately
    /// boils down to a single 3x3 matrix * vector3 multiplication. This should be preferred when available
    /// over the more generic `Color::convert`.
    pub fn convert_linear<DstSpace: LinearConvertFrom<SrcSpace>>(self) -> Color<DstSpace, St> {
        let conversion_mat =
            Mat3::from_cols_array(&<DstSpace as LinearConvertFrom<SrcSpace>>::MATRIX).transpose();
        Color::from(conversion_mat * self.raw)
    }
}

impl<SrcSpace, St> Color<SrcSpace, St> {
    /// Decodes `self` into the specified color space.
    pub fn decode<DstSpace: DecodeFrom<SrcSpace>>(self) -> Color<DstSpace, St> {
        Color::from(DstSpace::decode_raw(self.raw))
    }
}

impl<SrcSpace, St> Color<SrcSpace, St> {
    /// Encodes `self` into the specified color space.
    pub fn encode<DstSpace: EncodeFrom<SrcSpace>>(self) -> Color<DstSpace, St> {
        Color::from(DstSpace::encode_raw(self.raw))
    }
}

impl<Spc: LinearColorSpace, SrcSt> Color<Spc, SrcSt> {
    /// Converts this color from one state to another. This conversion is usecase and even instance dependent.
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
        Color::from(conversion_function(self.raw))
    }

    /// Changes this color's State. This assumes that you have done some kind of conversion externally,
    /// or that the proper conversion is simply a noop.
    pub fn cast_state<DstSt>(self) -> Color<Spc, DstSt> {
        Color::from(self.raw)
    }
}

impl<Spc: LinearColorSpace> Color<Spc, Scene> {
    /// Tonemap `self` using the `tonemapper`, converting `self` from being
    /// scene-referred to being display-referred.
    pub fn tonemap(self, tonemapper: impl Tonemapper) -> Color<Spc, Display> {
        Color::from(tonemapper.tonemap_raw(self.raw))
    }
}

impl<Spc: ColorSpace, St: State> Color<Spc, St> {
    /// Upcasts `self` into a [`DynamicColor`]
    pub fn dynamic(self) -> DynamicColor {
        DynamicColor {
            raw: self.raw,
            space: Spc::SPACE,
            state: St::STATE,
        }
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

/// A dynamic color, with its Space and State defined
/// as data. This is mostly useful for (de)serialization.
///
/// See [`ColorSpace`] and [`State`] for more.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct DynamicColor {
    /// The raw tristimulus value of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec3,
    pub space: DynamicColorSpace,
    pub state: DynamicState,
}

impl DynamicColor {
    pub fn new(raw: Vec3, space: DynamicColorSpace, state: DynamicState) -> Self {
        Self { raw, space, state }
    }

    /// Attempt to convert to a typed `Color`. Returns an error if `self`'s color space and state do not match
    /// the given types.
    pub fn downcast<Spc: ColorSpace, St: State>(self) -> ColorResult<Color<Spc, St>> {
        if self.space != Spc::SPACE {
            return Err(DowncastError::MismatchedSpace(self.space, Spc::SPACE).into());
        }

        if self.state != St::STATE {
            return Err(DowncastError::MismatchedState(self.state, St::STATE).into());
        }

        Ok(Color::from(self.raw))
    }

    /// Convert to a typed `Color` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    pub fn downcast_unchecked<Spc: ColorSpace, St: State>(self) -> Color<Spc, St> {
        Color::from(self.raw)
    }

    /// Convert `self` to the given color space. Must not attempt to convert to or from
    /// a nonlinear color space while in scene-referred state.
    pub fn convert(self, dest_space: DynamicColorSpace) -> Self {
        let conversion = kolor::ColorConversion::new(self.space, dest_space);
        let raw = conversion.convert(self.raw);
        Self {
            raw,
            space: dest_space,
            state: self.state,
        }
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
