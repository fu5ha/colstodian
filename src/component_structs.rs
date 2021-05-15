//! Structs that act as bags of named components which [`Color`][super::Color]s of different color spaces
//! may be `Deref`erenced to in order to gain more appropriate dot syntax for that color space.
//!
//! For example, a [`Color`][super::Color] in the [`ICtCpPQ`][super::ICtCpPQ] color space can be `Deref`'d to [`ICtCp`], allowing you
//! to do things like `color.i` or `color.ct`.

use core::fmt;

/// A bag of components with an alpha channel. Some `ColorAlpha`s will Deref
/// to this so that you can use `.alpha` and `.col.{component_name}`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColAlpha<T> {
    pub col: T,
    pub alpha: f32,
}

impl<T: fmt::Display> fmt::Display for ColAlpha<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, Alpha: {}", self.col, self.alpha)
    }
}

macro_rules! impl_bytemuck {
    ($($inner:ident),+) => {
        $(
            unsafe impl bytemuck::Zeroable for $inner {}
            unsafe impl bytemuck::Pod for $inner {}

            unsafe impl bytemuck::Zeroable for ColAlpha<$inner> {}
            unsafe impl bytemuck::Pod for ColAlpha<$inner> {}
        )+
    }
}

impl_bytemuck!(Rgb, ICtCp, Xyz, Lab, LCh);

/// A bag of components with names R, G, B. Some `Color`s with RGB color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R: {}, G: {}, B: {}", self.r, self.g, self.b)
    }
}

/// A bag of components with names I (Intensity), Ct (Chroma-Tritan), Cp (Chroma-Protan). Some `Color`s
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ICtCp {
    pub i: f32,
    pub ct: f32,
    pub cp: f32,
}

impl fmt::Display for ICtCp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "I: {}, Ct: {}, Cp: {}", self.i, self.ct, self.cp)
    }
}

/// A bag of components with names X, Y, Z. Some `Color`s with XYZ color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for Xyz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X: {}, Y: {}, Z: {}", self.x, self.y, self.z)
    }
}

/// A bag of components with names L (Luminance), a (green-red chroma), b (blue-yellow chroma).
/// Some `Color`s with XYZ color spaces will `Deref`/`DerefMut` to this struct so that you can
/// access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl fmt::Display for Lab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L: {}, a: {}, b: {}", self.l, self.a, self.b)
    }
}

/// A bag of components with names L (Luminance), C (chroma), h (hue).
/// Some `Color`s with XYZ color spaces will `Deref`/`DerefMut` to this struct so that you can
/// access their components with dot-syntax.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LCh {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl fmt::Display for LCh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L: {}, C: {}, h: {}", self.l, self.c, self.h)
    }
}
