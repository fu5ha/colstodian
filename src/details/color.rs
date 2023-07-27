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
use serde::{Deserialize, Serialize};

use core::fmt;
use core::ops::*;

/// A strongly typed color, parameterized by a [`ColorEncoding`]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "E::Repr: Serialize", deserialize = "E::Repr: Deserialize")))]
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

// impl<Spc, St> From<[f32; 3]> for Color<Spc, St> {
//     fn from(color: [f32; 3]) -> Self {
//         Self::new(color[0], color[1], color[2])
//     }
// }

// impl<Spc, St> AsRef<[f32; 3]> for Color<Spc, St> {
//     fn as_ref(&self) -> &[f32; 3] {
//         self.raw.as_ref()
//     }
// }
impl<E: ColorEncoding> Color<E> {
    /// Creates a [`Color`] from the raw data representation
    #[inline(always)]
    pub const fn from_repr(repr: E::Repr) -> Self {
        Self { repr }
    }
}

impl<SrcEnc: ColorEncoding> Color<SrcEnc> {
    /// Converts `self` from one color encoding to another.
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

impl<E> PartialEq for Color<E>
where
    E: ColorEncoding,
    E::Repr: PartialEq,
{
    #[inline]
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
