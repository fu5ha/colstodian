//! Structs that act as bags of named components which [`Color`][super::Color]s of different color spaces
//! may be `Deref`erenced to in order to gain more appropriate dot syntax for that color space.
//!
//! For example, a [`Color`][super::Color] in the [`ICtCpPQ`][super::ICtCpPQ] color space can be `Deref`'d to [`ICtCp`], allowing you
//! to do things like `color.i` or `color.ct`.

/// A bag of components with names R, G, B. Some `Color`s with RGB color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// A bag of components with names I (Intensity), Ct (Chroma-Tritan), Cp (Chroma-Protan). Some `Color`s
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct ICtCp {
    pub i: f32,
    pub ct: f32,
    pub cp: f32,
}

/// A bag of components with names X, Y, Z. Some `Color`s with XYZ color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A bag of components with names L (Luminance), a (green-red chroma), b (blue-yellow chroma).
/// Some `Color`s with XYZ color spaces will `Deref`/`DerefMut` to this struct so that you can
/// access their components with dot-syntax.
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

/// A bag of components with names L (Luminance), C (chroma), h (hue).
/// Some `Color`s with XYZ color spaces will `Deref`/`DerefMut` to this struct so that you can
/// access their components with dot-syntax.
pub struct LCh {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}
