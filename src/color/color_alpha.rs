use super::*;

use glam::Vec4Swizzles;

/// A strongly typed color with an alpha channel, parameterized by a color space and alpha state.
///
/// A color with an alpha channel is always in display-referred state. The alpha channel is always
/// linear [0..1].
///
/// See crate-level docs as well as [`ColorSpace`] and [`Alpha`] for more.
#[repr(C)]
#[derive(Derivative)]
#[derivative(Clone, Copy, PartialEq)]
pub struct ColorAlpha<Spc, A> {
    /// The raw values of the color. Be careful when modifying this directly.
    pub raw: Vec4,
    #[derivative(PartialEq = "ignore")]
    _pd: PhantomData<(Spc, A)>,
}

impl<Spc: ColorSpace, A: AlphaState> fmt::Display for ColorAlpha<Spc, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color<{}, {}>: ({})",
            Spc::default(),
            A::default(),
            self.deref()
        )
    }
}

impl<Spc: ColorSpace, A: AlphaState> fmt::Debug for ColorAlpha<Spc, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl<Spc, A> bytemuck::Zeroable for ColorAlpha<Spc, A> {}
#[cfg(feature = "bytemuck")]
unsafe impl<Spc, A> bytemuck::TransparentWrapper<Vec4> for ColorAlpha<Spc, A> {}
#[cfg(feature = "bytemuck")]
unsafe impl<Spc: 'static, A: 'static> bytemuck::Pod for ColorAlpha<Spc, A> {}

impl<Spc, A> ColorAlpha<Spc, A> {
    /// Creates a [`ColorAlpha`] with the raw internal color elements `el1`, `el2`, `el3` and alpha value `alpha`.
    #[inline]
    pub fn new(el1: f32, el2: f32, el3: f32, alpha: f32) -> Self {
        Self::from_raw(Vec4::new(el1, el2, el3, alpha))
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
}

/// Creates a [`ColorAlpha`] in the [`EncodedSrgb`] color space with components `r`, `g`, `b`, and `a`.
#[inline]
pub fn srgba<A: AlphaState>(r: f32, g: f32, b: f32, a: f32) -> ColorAlpha<EncodedSrgb, A> {
    ColorAlpha::new(r, g, b, a)
}

/// Creates a [`ColorAlpha`] in the [`LinearSrgb`] color space with components `r`, `g`, `b`, and `a`
#[inline]
pub fn linear_srgba<A: AlphaState>(r: f32, g: f32, b: f32, a: f32) -> ColorAlpha<LinearSrgb, A> {
    ColorAlpha::new(r, g, b, a)
}

impl<SrcSpace, SrcAlpha> ColorAlpha<SrcSpace, SrcAlpha>
where
    SrcSpace: ColorSpace,
    SrcAlpha: AlphaState,
{
    /// Converts from one color space to another. This is only implemented for the [`Separate`] alpha
    /// state because converting colors between nonlinear spaces with [`Premultiplied`] alpha is not a well-defined operation
    /// and will lead to odd behavior.
    pub fn convert<DstSpace, DstAlpha>(self) -> ColorAlpha<DstSpace, DstAlpha>
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

    /// Converts `self` to the provided `DstAlpha` [`AlphaState`].
    ///
    /// * If converting to the same state, this is a no-op.
    /// * If converting from [Premultiplied] to [Separate], you must ensure that `self.alpha != 0.0`, otherwise
    /// a divide by 0 will occur and `Inf`s will result.
    pub fn convert_alpha<DstAlpha: ConvertFromAlphaRaw<SrcAlpha> + AlphaState>(
        self,
    ) -> ColorAlpha<SrcSpace, DstAlpha> {
        let raw = self.raw.xyz();
        let alpha = self.raw.w;
        let converted = <DstAlpha as ConvertFromAlphaRaw<SrcAlpha>>::convert_raw(raw, alpha);
        ColorAlpha::from_raw(converted.extend(alpha))
    }

    /// Interprets this color as `DstSpace`. This assumes you have done an external computation/conversion such that this
    /// cast is valid.
    pub fn cast_space<DstSpace: ColorSpace>(self) -> ColorAlpha<DstSpace, SrcAlpha> {
        ColorAlpha::from_raw(self.raw)
    }

    /// Changes this color's alpha state. This assumes that you have done some kind of computation/conversion such that this
    /// cast is valid.
    pub fn cast_alpha_state<DstAlpha: AlphaState>(self) -> ColorAlpha<SrcSpace, DstAlpha> {
        ColorAlpha::from_raw(self.raw)
    }

    /// Changes this color's alpha state. This assumes that you have done some kind of computation/conversion such that this
    /// cast is valid.
    pub fn cast<DstSpace: ColorSpace, DstAlpha: AlphaState>(
        self,
    ) -> ColorAlpha<DstSpace, DstAlpha> {
        ColorAlpha::from_raw(self.raw)
    }
}

impl<Spc: NonlinearColorSpace, A: AlphaState> ColorAlpha<Spc, A> {
    /// Convert `self` into the closest linear color space.
    pub fn linearize(self) -> ColorAlpha<Spc::LinearSpace, A> {
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

impl<Spc> ColorAlpha<Spc, Separate> {
    /// Converts `self` to a [`Color`] by stripping off the alpha component.
    pub fn into_color_no_premultiply(self) -> Color<Spc, Display> {
        Color::from_raw(self.raw.xyz())
    }
}

impl<Spc> ColorAlpha<Spc, Premultiplied> {
    /// Converts `self` to a [`Color`] by stripping off the alpha component.
    pub fn into_color(self) -> Color<Spc, Display> {
        Color::from_raw(self.raw.xyz())
    }
}

impl<Spc: AsU8Array, A: AlphaState> ColorAlpha<Spc, A> {
    /// Convert `self` to a `[u8; 4]`. All components of `self` *must* be in range `[0..1]`.
    pub fn to_u8(self) -> [u8; 4] {
        fn f32_to_u8(x: f32) -> u8 {
            (x * 255.0).round() as u8
        }
        [
            f32_to_u8(self.raw.x),
            f32_to_u8(self.raw.y),
            f32_to_u8(self.raw.z),
            f32_to_u8(self.raw.w),
        ]
    }

    /// Decode a `[u8; 4]` into a `ColorAlpha` with specified space and alpha state.
    pub fn from_u8(encoded: [u8; 4]) -> ColorAlpha<Spc, A> {
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

macro_rules! impl_op_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, A> $op for ColorAlpha<Spc, A> {
            type Output = ColorAlpha<Spc, A>;
            fn $op_func(self, rhs: ColorAlpha<Spc, A>) -> Self::Output {
                ColorAlpha::from_raw(self.raw.$op_func(rhs.raw))
            }
        }
    };
}

macro_rules! impl_op_color_float {
    ($op:ident, $op_func:ident) => {
        impl<Spc: LinearColorSpace, A> $op<f32> for ColorAlpha<Spc, A> {
            type Output = ColorAlpha<Spc, A>;
            fn $op_func(self, rhs: f32) -> Self::Output {
                ColorAlpha::from_raw(self.raw.$op_func(rhs))
            }
        }

        impl<Spc: LinearColorSpace, A> $op<ColorAlpha<Spc, A>> for f32 {
            type Output = ColorAlpha<Spc, A>;
            fn $op_func(self, rhs: ColorAlpha<Spc, A>) -> Self::Output {
                ColorAlpha::from_raw(self.$op_func(rhs.raw))
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

/// A dynamic color with an alpha channel, with its space and alpha defined
/// as data. This is mostly useful for (de)serialization.
///
/// See [`ColorAlpha`], [`ColorSpace`] and [`AlphaState`] for more.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct DynamicColorAlpha {
    /// The raw tristimulus value of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec4,
    pub space: DynamicColorSpace,
    pub alpha_state: DynamicAlphaState,
}

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
        let color_alpha = self.premultiply();
        DynamicColor::new(color_alpha.raw.xyz(), self.space, DynamicState::Display)
    }

    /// Converts `self` to a [`DynamicColor`] by stripping off the alpha component, without checking
    /// whether it is premultiplied or not.
    pub fn into_color_no_premultiply(self) -> DynamicColor {
        DynamicColor::new(self.raw.xyz(), self.space, DynamicState::Display)
    }

    /// Convert `self` to the given color space. Must not attempt to convert to or from
    /// a nonlinear color space while in [Premultiplied][DynamicAlphaState::Premultiplied] alpha state.
    pub fn convert(self, dest_space: DynamicColorSpace) -> ColorResult<Self> {
        if self.alpha_state == DynamicAlphaState::Premultiplied
            && (!self.space.is_linear() || !dest_space.is_linear())
        {
            return Err(
                DynamicConversionError::NonlinearConversionInPremultipliedAlphState(
                    self.space, dest_space,
                )
                .into(),
            );
        }
        let conversion = kolor::ColorConversion::new(self.space, dest_space);
        let raw = conversion.convert(self.raw.xyz()).extend(self.raw.w);
        Ok(Self {
            raw,
            space: dest_space,
            alpha_state: self.alpha_state,
        })
    }

    /// Premultiply `self`'s first three components with its alpha, resulting in a color with [Premultiplied][DynamicAlphaState::Premultiplied] alpha.
    ///
    /// If `self` is already in [Premultiplied][DynamicAlphaState::Premultiplied] alpha state, this does nothing.
    pub fn premultiply(self) -> DynamicColorAlpha {
        let col = if self.alpha_state == DynamicAlphaState::Separate {
            self.raw.xyz() * self.raw.w
        } else {
            self.raw.xyz()
        };

        Self {
            raw: col.extend(self.raw.w),
            space: self.space,
            alpha_state: DynamicAlphaState::Premultiplied,
        }
    }

    /// The inverse of [`DynamicColorAlpha::premultiply`]. Divides `self`'s first three components by its alpha,
    /// resulting in a color with [Separate][DynamicAlphaState::Separate] alpha.
    ///
    /// This operation does nothing if `self`'s alpha is 0.0 or if `self` is already in [Premultiplied][DynamicAlphaState::Separate] alpha state.
    pub fn separate(self) -> DynamicColorAlpha {
        let col = if self.alpha_state == DynamicAlphaState::Premultiplied && self.raw.w != 0.0 {
            self.raw.xyz() / self.raw.w
        } else {
            self.raw.xyz()
        };

        Self {
            raw: col.extend(self.raw.w),
            space: self.space,
            alpha_state: DynamicAlphaState::Separate,
        }
    }
}

/// An object-safe trait implemented by both [`ColorAlpha`] and [`DynamicColorAlpha`].
pub trait AnyColorAlpha {
    fn raw(&self) -> Vec4;
    fn space(&self) -> DynamicColorSpace;
    fn alpha_state(&self) -> DynamicAlphaState;

    /// Upcasts `self` into a [`DynamicColorAlpha`]
    fn dynamic(&self) -> DynamicColorAlpha {
        DynamicColorAlpha::new(self.raw(), self.space(), self.alpha_state())
    }
}

impl<'a> From<&'a dyn AnyColorAlpha> for DynamicColorAlpha {
    fn from(color: &'a dyn AnyColorAlpha) -> DynamicColorAlpha {
        color.dynamic()
    }
}

/// A type that implements this trait provides the ability to downcast from a dynamically-typed
/// color to a statically-typed [`ColorAlpha`]. This is implemented for all types that implement [`AnyColorAlpha`]
pub trait DynColorAlpha {
    /// Attempt to convert to a typed [`ColorAlpha`]. Returns an error if `self`'s color space and alpha state do not match
    /// the given types.
    fn downcast<Spc: ColorSpace, A: AlphaState>(&self) -> ColorResult<ColorAlpha<Spc, A>>;

    /// Convert to a typed `ColorAlpha` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    fn downcast_unchecked<Spc: ColorSpace, A: AlphaState>(&self) -> ColorAlpha<Spc, A>;
}

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
