use crate::traits::*;

/*
#[cfg(not(target_arch = "spirv"))]
use crate::{
    error::{DowncastError, DynamicConversionError},
    ColorResult,
};
*/

use glam::Vec3;
use glam::Vec4;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use core::fmt;
use core::ops::*;

/// A strongly typed color, parameterized by a [`ColorEncoding`].
///
/// [`Color`] is a unified type that defines a color in any [`ColorEncoding`].
/// The [`ColorEncoding`] defines a bunch of different properties about how the
/// color values are stored and what those values actually mean. For example,
/// [`Color<SrgbU8>`] is a color with red, green, and blue values that vary from
/// `0-255` and the meaning of those values is defined by the full sRGB color encoding standard.
/// The most common and standard color encodings are exposed in the
/// [`basic_encodings`][crate::basic_encodings] module.
///
/// To create a new [`Color`] value, see the list of constructor helpers in the docs below.
///
/// There are several ways to work with an existing color. First, you can always access the raw data
/// in the encoding's [`Repr`][ColorEncoding::Repr] directly by accessing `col.repr`. You can also always
/// access the individual named color components through dot-syntax because of the `Deref` and `DerefMut`
/// impls to the encoding's [`ComponentStruct`][ColorEncoding::ComponentStruct]. For example, in an RGB color space,
/// you can access the components with `.r`, `.g`, and `.b`.
///
/// ```
/// # use colstodian::Color;
/// # use colstodian::basic_encodings::{LinearSrgb, SrgbU8};
/// # use glam::Vec3;
///
/// let col: Color<SrgbU8> = Color::srgb_u8(100u8, 105u8, 220u8);
///
/// assert_eq!(col.repr, [100u8, 105u8, 220u8]);
///
/// let mut col2: Color<LinearSrgb> = Color::linear_srgb(0.5, 1.0, 0.25);
///
/// assert_eq!(col2.repr, Vec3::new(0.5, 1.0, 0.25));
///
/// col2.b = 0.75;
///
/// assert_eq!(col2.r, 0.5);
/// assert_eq!(col2.g, 1.0);
/// assert_eq!(col2.b, 0.75);
/// ```
///
/// In order to do math on color types without accessing the underlying repr directly, you'll need
/// to be in a [`WorkingEncoding`], which is a trait implemented by encodings that support doing
/// such math operations well.
///
/// You can convert between color encodings using the [`.convert::<E>()`][Color::convert] method.
///
/// ### Basic Conversion Example
///
/// Here we construct two colors in different ways, convert them both to [`LinearSrgb`] to work with them,
/// and then convert the resul to [`SrgbU8`] which can be passed on to be displayed in an image.
///
/// ```
/// use colstodian::Color;
/// use colstodian::basic_encodings::{SrgbU8, LinearSrgb};
///
/// let color1 = Color::srgb_u8(102, 54, 220);
/// let color2 = Color::srgb_f32(0.5, 0.8, 0.1);
///
/// let color1_working = color1.convert::<LinearSrgb>();
/// let color2_working = color2.convert::<LinearSrgb>();
///
/// let result_working = color1_working * 0.5 + color2_working;
///
/// let output = result_working.convert::<SrgbU8>();
///
/// assert_eq!(output, Color::srgb_u8(144, 206, 163));
/// ```
///
/// [`LinearSrgb`]: crate::details::encodings::LinearSrgb
/// [`SrgbU8`]: crate::details::encodings::SrgbU8
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E::Repr: Serialize",
        deserialize = "E::Repr: DeserializeOwned"
    ))
)]
pub struct Color<E: ColorEncoding> {
    /// The raw values of the color. Be careful when modifying this directly.
    pub repr: E::Repr,
}

impl<E: ColorEncoding> Copy for Color<E> {}

impl<E: ColorEncoding> Clone for Color<E> {
    #[inline(always)]
    fn clone(&self) -> Color<E> {
        *self
    }
}

impl<E: ColorEncoding> Color<E> {
    /// Creates a [`Color`] from the raw data representation of the specified color encoding.
    #[inline(always)]
    pub const fn from_repr(repr: E::Repr) -> Self {
        Self { repr }
    }
}

impl<SrcEnc: ColorEncoding> Color<SrcEnc> {
    /// Converts `self` from one color encoding to another.
    ///
    /// In order to be able to [`convert`][Color::convert] from `EncodingA` to `EncodingB`, `EncodingB`
    /// must implement [`ConvertFrom<EncodingA>`].
    ///
    /// If that trait is not implemented for a pair of encodings, then a direct conversion without input or choice from the user
    /// is not possible, and a conversion between the encodings will need to be performed manually or in more than one step.
    ///
    /// If you are able to [`convert`][Color::convert] from `EncodingA` to `EncodingB`, then you can also use a
    /// `Color<EncodingA>` anywhere you need a type that implements [`ColorInto<Color<EncodingB>>`][crate::ColorInto]!
    ///
    /// ## Example
    ///
    /// ```
    /// # use colstodian::*;
    /// # use colstodian::basic_encodings::*;
    /// # use colstodian::equals_eps::*;
    /// let grey_f32 = Color::srgb_f32(0.5, 0.5, 0.5);
    /// let grey_u8 = Color::srgb_u8(127, 127, 127);
    ///
    /// assert_eq_eps!(grey_f32.convert::<SrgbU8>(), grey_u8, 0);
    ///
    /// let col = Color::srgb_u8(102, 51, 153);
    /// let col_linear_srgb = col.convert::<LinearSrgb>();
    ///
    /// assert_eq_eps!(col_linear_srgb, Color::linear_srgb(0.13287, 0.0331, 0.31855), 0.0001);
    /// ```
    pub fn convert<DstEnc>(self) -> Color<DstEnc>
    where
        DstEnc: ColorEncoding + ConvertFrom<SrcEnc>,
        DstEnc::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
    {
        let mut repr = self.repr;

        // src conversion map
        <DstEnc as ConvertFrom<SrcEnc>>::map_src(&mut repr);

        // src transform
        let (mut raw, alpha) = SrcEnc::src_transform_raw(self.repr);

        // linear part
        <DstEnc::LinearSpace as LinearConvertFromRaw<SrcEnc::LinearSpace>>::linear_part_raw(
            &mut raw,
        );

        // dst transform
        let dst_repr = DstEnc::dst_transform_raw(raw, alpha);

        Color::from_repr(dst_repr)
    }

    /// Interprets this color as `DstEnc`. Requires that `DstEnc`'s `ColorEncoding::Repr` is the same as `self`'s.
    ///
    /// Using this method assumes you have done an external computation/conversion such that this cast is valid.
    #[inline(always)]
    pub fn cast<DstEnc: ColorEncoding<Repr = SrcEnc::Repr>>(self) -> Color<DstEnc> {
        Color { repr: self.repr }
    }
}

impl<E: ColorEncoding + Saturate> Color<E> {
    /// Clamp the raw element values of `self` within the current color encoding's valid range of values.
    #[inline]
    pub fn saturate(self) -> Self {
        Self::from_repr(<E as Saturate>::saturate(self.repr))
    }
}

impl<E> Color<E>
where
    E: ColorEncoding + AlphaOver,
{
    /// Alpha-composite `self` over `under`.
    #[inline(always)]
    pub fn alpha_over(self, under: Self) -> Color<E> {
        <E as AlphaOver>::composite(self, under)
    }
}

impl<E> Color<E>
where
    E: ColorEncoding + PerceptualEncoding + LinearInterpolate,
    E::Repr: Add<Output = E::Repr> + Sub<Output = E::Repr> + Mul<f32, Output = E::Repr>,
{
    /// Blend `self`'s color values with the color values from `other` with perceptually-linear interpolation.
    ///
    /// `factor` ranges from `[0..=1.0]`. If `factor` is > `1.0`, results may not be sensical.
    #[inline]
    pub fn perceptual_blend(self, other: Color<E>, factor: f32) -> Color<E> {
        self.lerp(other, factor)
    }
}

impl<E> Color<E>
where
    E: ColorEncoding + LinearInterpolate,
    E::Repr: Add<Output = E::Repr> + Sub<Output = E::Repr> + Mul<f32, Output = E::Repr>,
{
    /// Linearly interpolate from `self`'s value to `other`'s value. Not guaranteed to be perceptually
    /// linear or pleasing!
    ///
    /// If you want a better way to blend colors in a perceptually pleasing way, see [`Color::perceptual_blend`],
    /// which requires that the color encoding is a [`PerceptualEncoding`].
    ///
    /// `factor` ranges from `[0..=1.0]`. If `factor` is > `1.0`, results may not be sensical.
    #[inline]
    pub fn lerp(self, other: Self, factor: f32) -> Self {
        <E as LinearInterpolate>::lerp(self, other, factor)
    }
}

impl<T, E> AsRef<T> for Color<E>
where
    E: ColorEncoding,
    E::Repr: AsRef<T>,
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        self.repr.as_ref()
    }
}

impl<E> PartialEq for Color<E>
where
    E: ColorEncoding,
    E::Repr: PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &Color<E>) -> bool {
        self.repr == other.repr
    }
}

// SAFETY: Color is transparent with the underlying repr
#[cfg(feature = "bytemuck")]
unsafe impl<E> bytemuck::Zeroable for Color<E>
where
    E: ColorEncoding,
    E::Repr: bytemuck::Zeroable,
{
}

// SAFETY: Color is transparent with the underlying repr
#[cfg(feature = "bytemuck")]
unsafe impl<E> bytemuck::Pod for Color<E>
where
    E: ColorEncoding,
    E::Repr: bytemuck::Pod,
{
}

// SAFETY: Color is transparent with the underlying repr
#[cfg(feature = "bytemuck")]
unsafe impl<E: ColorEncoding> bytemuck::TransparentWrapper<E::Repr> for Color<E> {}

impl<E: ColorEncoding> Deref for Color<E> {
    type Target = E::ComponentStruct;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        <E::ComponentStruct as ComponentStructFor<E::Repr>>::cast(&self.repr)
    }
}

impl<E: ColorEncoding> DerefMut for Color<E> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        <E::ComponentStruct as ComponentStructFor<E::Repr>>::cast_mut(&mut self.repr)
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<E> fmt::Debug for Color<E>
where
    E: ColorEncoding,
    E::ComponentStruct: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color<{}>({:?})", E::NAME, <Self as Deref>::deref(self))
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<E> fmt::Display for Color<E>
where
    E: ColorEncoding,
    E::ComponentStruct: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color<{}>({})", E::NAME, <Self as Deref>::deref(self))
    }
}

// --------- MATH OP IMPLS -----------
//
// For working encodings, we want to be able to multiply or divide by
// a unitless quantity of the same shape as the underlying representation
// or element type as a scaling factor.
//
// `col * Repr`
// `col * Repr::Element`
// `Repr * col`
// `Repr::Element * col`
// `col *= Repr`
// `col *= Repr::Element`
//
// `col / Repr`
// `col / Repr::Element`
// `Repr / col`
// `Repr::Element / col`
// `col /= Repr`
// `col /= Repr::Element`
//
// We don't want to be able to multiply or divide a color by another color
// of the same encoding because then we'd just end up with a unitless ratio.
// If someone wants such a quantity, they can access the underlying data and
// do componentwise division themselves, but the fact such an operation is not
// implemented directly on the color type may give pause that the operation is often
// nonsensical.
//
// we also want to be able to add and subtract colors with the same encoding directly
//
// `col + col`
// `col += col`
//
// `col - col`
// `col - col`

impl<Rhs, E> Mul<Rhs> for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Mul<Rhs, Output = E::Repr>,
{
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Rhs) -> Self::Output {
        Self {
            repr: self.repr.mul(rhs),
        }
    }
}

impl<Rhs, E> MulAssign<Rhs> for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: MulAssign<Rhs>,
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Rhs) {
        self.repr.mul_assign(rhs)
    }
}

impl<E> Mul<Color<E>> for f32
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Mul<f32, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn mul(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.mul(self);
        rhs
    }
}

impl<E> Mul<Color<E>> for Vec3
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Mul<Vec3, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn mul(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.mul(self);
        rhs
    }
}

impl<E> Mul<Color<E>> for Vec4
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Mul<Vec4, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn mul(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.mul(self);
        rhs
    }
}

impl<Rhs, E> Div<Rhs> for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Div<Rhs, Output = E::Repr>,
{
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Rhs) -> Self::Output {
        Self {
            repr: self.repr.div(rhs),
        }
    }
}

impl<Rhs, E> DivAssign<Rhs> for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: DivAssign<Rhs>,
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Rhs) {
        self.repr.div_assign(rhs)
    }
}

impl<E> Div<Color<E>> for f32
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Div<f32, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn div(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.div(self);
        rhs
    }
}

impl<E> Div<Color<E>> for Vec3
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Div<Vec3, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn div(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.div(self);
        rhs
    }
}

impl<E> Div<Color<E>> for Vec4
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Div<Vec4, Output = E::Repr>,
{
    type Output = Color<E>;
    #[inline(always)]
    fn div(self, mut rhs: Color<E>) -> Self::Output {
        rhs.repr = rhs.repr.div(self);
        rhs
    }
}

impl<E> Add for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Add<Output = E::Repr>,
{
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Color<E>) -> Self::Output {
        Self {
            repr: self.repr.add(rhs.repr),
        }
    }
}

impl<E> AddAssign for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: AddAssign,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.repr.add_assign(rhs.repr)
    }
}

impl<E> Sub for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Sub<Output = E::Repr>,
{
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Color<E>) -> Self::Output {
        Self {
            repr: self.repr.sub(rhs.repr),
        }
    }
}

impl<E> SubAssign for Color<E>
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: SubAssign,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.repr.sub_assign(rhs.repr)
    }
}

// --------- END MATH OP IMPLS -----------
