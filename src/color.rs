use crate::traits::*;

/*
#[cfg(not(target_arch = "spirv"))]
use crate::{
    error::{DowncastError, DynamicConversionError},
    ColorResult,
};
*/

use glam::Vec3;
#[cfg(all(not(feature = "std"), feature = "libm"))]
use num_traits::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt;
use core::ops::*;

/// A strongly typed color, parameterized by a color space and state.
///
/// See crate-level docs as well as [`ColorSpace`] and [`State`] for more.
#[repr(transparent)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Color<E: ColorEncoding> {
    /// The raw values of the color. Be careful when modifying this directly.
    pub repr: E::Repr,
}

#[macro_export]
macro_rules! const_color {
    ($el1:expr, $el2:expr, $el3:expr) => {
        Color::from_raw(const_vec3!([$el1, $el2, $el3]))
    };
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

impl<E: ColorEncoding + Saturate> Color<E> {
    /// Clamp the raw element values of `self` in the range [0..1]
    #[inline]
    pub fn saturate(self) -> Self {
        Self::from_repr(<E as Saturate>::saturate(self.repr))
    }
}

impl<SrcEnc: ColorEncoding> Color<SrcEnc> {
    /// Converts `self` from one color encoding to another.
    pub fn convert<DstEnc>(self) -> Color<DstEnc>
    where
        DstEnc: ColorEncoding,
        DstEnc::LinearSpace: ConvertFromRaw<SrcEnc::LinearSpace>,
    {
        let (mut raw, alpha) = SrcEnc::src_transform_raw(self.repr);
        <DstEnc::LinearSpace as ConvertFromRaw<SrcEnc::LinearSpace>>::linear_part_raw(&mut raw);
        Color::from_repr(DstEnc::dst_transform_raw(raw, alpha))
    }

    /// Interprets this color as `DstEnc`. Requires that `DstEnc`'s `ColorEncoding::Repr` is the same as `self`'s.
    /// 
    /// Using this method assumes you have done an external computation/conversion such that this cast is valid.
    #[inline(always)]
    pub fn cast<DstEnc: ColorEncoding<Repr = SrcEnc::Repr>>(self) -> Color<DstEnc> {
        Color { repr: self.repr }
    }
}

impl<E> Color<E>
where
    E: ColorEncoding + Blend,
{
    /// Blend `self`'s color values with the color values from `other` with linear interpolation. If `factor` is > 1.0,
    /// results may not be sensical.
    #[inline]
    pub fn blend(self, other: Color<E>, factor: f32) -> Color<E> {
        Color::from_repr(<E as Blend>::blend(self.repr, other.repr, factor))
    }
}

impl<E: ColorEncoding> Copy for Color<E> {}

impl<E: ColorEncoding> Clone for Color<E> {
    #[inline(always)]
    fn clone(&self) -> Color<E> {
        *self
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

#[cfg(feature = "bytemuck")]
unsafe impl<E: ColorEncoding> bytemuck::Zeroable for Color<E> {}
#[cfg(feature = "bytemuck")]
unsafe impl<E: ColorEncoding> bytemuck::TransparentWrapper<E::Repr> for Color<E> {}
#[cfg(feature = "bytemuck")]
unsafe impl<E: ColorEncoding> bytemuck::Pod for Color<E> {}

#[cfg(not(target_arch = "spirv"))]
impl<E> fmt::Display for Color<E>
where
    E: ColorEncoding + fmt::Display,
    E::ComponentStruct: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color<{}>({})",
            E::NAME,
            <Self as Deref>::deref(self)
        )
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
    E: ColorEncoding + fmt::Display,
    E::ComponentStruct: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color<{}>({})",
            E::NAME,
            <Self as Deref>::deref(self)
        )
    }
}

// macro_rules! impl_op_color {
//     ($op:ident, $op_func:ident) => {
//         impl<Spc: LinearColorSpace, St> $op for Color<Spc, St> {
//             type Output = Color<Spc, St>;
//             fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
//                 Color::from_raw(self.raw.$op_func(rhs.raw))
//             }
//         }

//         impl<Spc: LinearColorSpace, St> $op<Vec3> for Color<Spc, St> {
//             type Output = Color<Spc, St>;
//             fn $op_func(self, rhs: Vec3) -> Self::Output {
//                 Color::from_raw(self.raw.$op_func(rhs))
//             }
//         }
//     };
// }

// macro_rules! impl_binop_color {
//     ($op:ident, $op_func:ident) => {
//         impl<Spc: LinearColorSpace, St> $op for Color<Spc, St> {
//             fn $op_func(&mut self, rhs: Color<Spc, St>) {
//                 self.raw.$op_func(rhs.raw);
//             }
//         }

//         impl<Spc: LinearColorSpace, St> $op<Vec3> for Color<Spc, St> {
//             fn $op_func(&mut self, rhs: Vec3) {
//                 self.raw.$op_func(rhs);
//             }
//         }
//     };
// }

// impl<E> Div<E::Element> for Color<E>
// where
//     E: ColorEncoding + WorkingEncoding,
//     E::Repr: Div<E::Element>,
// {
//     type Output = Self;
//     #[inline]
//     fn div(self, rhs: <E::Repr as ColorRepr>::Element) -> Self::Output {
//         Color {
//             repr: self.repr.div(rhs),
//         }
//     }
// }
// impl<E> Div<Color<E>> for E::Element
// where
//     E: ColorEncoding + WorkingEncoding,
//     E::Repr: Div<E::Element>,
// {
//     type Output = Color<E>;
//     #[inline]
//     fn div(self, rhs: Color<E>) -> Self::Output {
//         Color {
//             repr: rhs.repr.div(self)
//         }
//     }
// }

// impl_op_color!(Add, add);
// impl_op_color!(Sub, sub);
// impl_op_color!(Mul, mul);
// impl_op_color!(Div, div);

// impl_binop_color!(AddAssign, add_assign);
// impl_binop_color!(SubAssign, sub_assign);
// impl_binop_color!(MulAssign, mul_assign);
// impl_binop_color!(DivAssign, div_assign);
