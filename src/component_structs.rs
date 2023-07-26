//! Structs that act as bags of named components which [`Color`][super::Color]s of different color spaces
//! may be `Deref`erenced to in order to gain more appropriate dot syntax for that color space.
//!
//! For example, a [`Color`][super::Color] in the [`ICtCpPQ`][super::ICtCpPQ] color space can be `Deref`'d to [`ICtCp`], allowing you
//! to do things like `color.i` or `color.ct`.

use core::fmt;

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

#[cfg(not(target_arch = "spirv"))]
impl<T: fmt::Display> fmt::Display for Rgb<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R: {}, G: {}, B: {}", self.r, self.g, self.b)
    }
}
