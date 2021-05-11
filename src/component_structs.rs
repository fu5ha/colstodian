//! Structs that act as bags of named components which [`Color`][super::Color]s of different color spaces
//! may be `Deref`erenced to in order to gain more appropriate dot syntax for that color space.
//!
//! For example, a [`Color`] in the [`ICtCpPQ`] color space can be `Deref`'d to [`ICtCp`], allowing you
//! to do things like `color.i` or `color.ct`.

/// A bag of components with names R, G, B. Some `Color`s with RGB color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// A bag of components with names I, Ct, Cp. Some `Color`s with ICtCp color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct ICtCp {
    pub i: f32,
    pub ct: f32,
    pub cp: f32,
}
