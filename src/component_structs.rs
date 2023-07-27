//! Structs that act as bags of named components which [`Color`][super::Color]s of different color spaces
//! may be `Deref`erenced to in order to gain more appropriate dot syntax for that color space.
//!
//! For example, a [`Color`][super::Color] in the [`ICtCpPQ`][super::ICtCpPQ] color space can be `Deref`'d to [`ICtCp`], allowing you
//! to do things like `color.i` or `color.ct`.

use core::fmt;

use crate::reprs::*;
use crate::traits::ComponentStructFor;

// #[cfg(feature = "bytemuck")]
// macro_rules! impl_bytemuck {
//     ($($inner:ident),+) => {
//         $(
//             unsafe impl bytemuck::Zeroable for $inner {}
//             unsafe impl bytemuck::Pod for $inner {}

//             unsafe impl bytemuck::Zeroable for ColAlpha<$inner> {}
//             unsafe impl bytemuck::Pod for ColAlpha<$inner> {}
//         )+
//     }
// }

// #[cfg(feature = "bytemuck")]
// impl_bytemuck!(Rgb, ICtCp, Xyz, Lab, LCh);

/// A bag of components with names R, G, B. Some `Color`s with RGB color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

unsafe impl ComponentStructFor<U8Repr> for Rgb<u8> {
    fn cast(repr: &U8Repr) -> &Self {
        // SAFETY: [u8; 3] is guaranteed to have the same layout as Self
        unsafe { &*(repr as *const U8Repr as *const Self ) }
    }

    fn cast_mut(repr: &mut U8Repr) -> &mut Self {
        // SAFETY: [u8; 3] is guaranteed to have the same layout as Self
        unsafe { &mut *(repr as *mut U8Repr as *mut Self ) }
    }
}

unsafe impl ComponentStructFor<F32Repr> for Rgb<f32> {
    fn cast(repr: &F32Repr) -> &Self {
        // SAFETY: Vec3 is guaranteed to have the same layout as Self
        unsafe { &*(repr as *const F32Repr as *const Self ) }
    }

    fn cast_mut(repr: &mut F32Repr) -> &mut Self {
        // SAFETY: Vec3 is guaranteed to have the same layout as Self
        unsafe { &mut *(repr as *mut F32Repr as *mut Self ) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<T: fmt::Display> fmt::Display for Rgb<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R: {}, G: {}, B: {}", self.r, self.g, self.b)
    }
}
/// A bag of components with names R, G, B, A. Some `Color`s with RGBA color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct RgbA<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

unsafe impl ComponentStructFor<U8ARepr> for RgbA<u8> {
    fn cast(repr: &U8ARepr) -> &Self {
        // SAFETY: [u8; 4] is guaranteed to have the same layout as Self
        unsafe { &*(repr as *const U8ARepr as *const Self ) }
    }

    fn cast_mut(repr: &mut U8ARepr) -> &mut Self {
        // SAFETY: [u8; 4] is guaranteed to have the same layout as Self
        unsafe { &mut *(repr as *mut U8ARepr as *mut Self ) }
    }
}

unsafe impl ComponentStructFor<F32ARepr> for RgbA<f32> {
    fn cast(repr: &F32ARepr) -> &Self {
        // SAFETY: Vec4 is guaranteed to have the same layout as Self
        unsafe { &*(repr as *const F32ARepr as *const Self ) }
    }

    fn cast_mut(repr: &mut F32ARepr) -> &mut Self {
        // SAFETY: Vec4 is guaranteed to have the same layout as Self
        unsafe { &mut *(repr as *mut F32ARepr as *mut Self ) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<T: fmt::Display> fmt::Display for RgbA<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R: {}, G: {}, B: {}", self.r, self.g, self.b)
    }
}
