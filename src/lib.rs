//! An opinionated color management library for games and graphics.
//!
//! # Introduction
//!
//! Color is a really complex and fascinating topic, and I'd love to take you on a little tour
//! and show you how `colstodian` tries to help make it easier to manage. But, if you really just want to get
//! sh\*t done right now, here's the basics:
//!
//! [`Color`] is a unified type representing a color in any [`ColorEncoding`].
//! The [`ColorEncoding`] defines a bunch of different properties about how the
//! color values are stored and what those values actually mean. For example,
//! [`Color<SrgbU8>`] is a color with red, green, and blue values that vary from
//! `0-255` and the meaning of those values is defined by the full sRGB color encoding standard.
//! The most common and standard color encodings are exposed in the [`basic_encodings`] module.
//!
//! Many built-in color encodings expose constructor functions on the [`Color`]
//! type. See [the docs for that type][Color] for a full list. The ones you are likely most interested in
//! if you don't know much about color are:
//!
//! * [`Color::srgb_u8`]: If you have three `0-255` values, this is what you want
//! * [`Color::srgb_f32`]: If you have three `0.0..=1.0` values, this is probably what you want
//! * [`Color::linear_srgb`]: If you have three `0.0..=1.0` values that you know are "linear rgb", this is probably what you want
//!
//! If you also have alpha (i.e. four values instead of three), then [`Color::srgba_u8`], [`Color::srgba_f32`], and [`Color::linear_srgba`]
//! are the equivalents of the above with an alpha component.
//!
//! If you want to do math to a color (for example, adding two colors together or multiplying one by a coefficient),
//! you'll want to do so in a color encoding that is conducive to that. Color encodings which have this property implement
//! the [`WorkingEncoding`][details::traits::WorkingEncoding] trait. If a [`Color`] is not encoded in a working encoding,
//! it will not implement common math traits like addition, multiplication, etc.
//!
//! The most common [`WorkingEncoding`] is [`LinearSrgb`]. You can convert a color you have created using any of the
//! constructors above to [`LinearSrgb`] by using the [`.convert::<E>()`][Color::convert] method.
//!
//! If you want to output a color into an image file, the most common color encoding for most common image formats
//! (and the one assumed by viewing programs if a color profile is not embedded) is [`SrgbU8`].
//! You can convert a color from a working encoding to [`SrgbU8`] for output again with the [`.convert::<E>()`][Color::convert]
//! method.
//!
//! ### Example
//!
//! Here we construct two colors in different ways, convert them both to [`LinearSrgb`] to work with them, and then convert the result
//! to [`SrgbU8`] which can be passed on to be displayed in an image.
//!
//! ```
//! use colstodian::Color;
//! use colstodian::basic_encodings::{SrgbU8, LinearSrgb};
//!
//! let color1 = Color::srgb_u8(102, 54, 220);
//! let color2 = Color::srgb_f32(0.5, 0.8, 0.1);
//!
//! let color1_working = color1.convert::<LinearSrgb>();
//! let color2_working = color2.convert::<LinearSrgb>();
//!
//! let result_working = color1_working * 0.5 + color2_working;
//!
//! let output = result_working.convert::<SrgbU8>();
//!
//! assert_eq!(output, Color::srgb_u8(144, 206, 163));
//! ```
//!
//! ## Color Encoding Basics
//!
//! Much like how a 3d vector like a `glam::Vec3` could be used to describe any of:
//!
//! * The motion vector of an object in meters per second
//! * The position of an object relative to a reference point in kilometers
//! * Three "wellness scores" for a character, which each axis representing how happy the character
//!   is about some aspect of their life
//!
//! A bag of components that describes "a color" could actually be interpreted in many different
//! ways, and the end result of what those components mean is very different.
//!
//! `colstodian` gathers all the information that defines how a color is represented in data as well as
//! what that data actually means into representative types that implement the [`ColorEncoding`] trait.
//!
//! [`LinearSrgb`]: details::encodings::LinearSrgb
//! [`SrgbU8`]: details::encodings::SrgbU8
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(
    clippy::let_and_return, // it makes conversion code more explicit with naming
)]

/// Contains advanced usage details of the crate.
pub mod details {
    pub mod component_structs;

    /// Types representing different [`ColorEncoding`][traits::ColorEncoding]s.
    pub mod encodings;

    /// Contains the [`Color`][color::Color] type and helper functions.
    pub mod color;

    /// Types representing different [`LinearColorSpace`][traits::LinearColorSpace]s.
    #[rustfmt::skip]
    pub mod linear_spaces;

    /// The traits which form the backbone of this crate.
    pub mod traits;

    /// The underlying data representations ([`ColorRepr`][traits::ColorRepr]s) used by different [`ColorEncoding`][traits::ColorEncoding]s.
    pub mod reprs;
}

pub(crate) use details::*;

/// Contains a basic set of [`ColorEncoding`]s to get most people going.
///
/// These are all re-exported from inside the [`details::encodings`]
pub mod basic_encodings {
    #[doc(inline)]
    pub use crate::details::encodings::LinearSrgb;
    #[doc(inline)]
    pub use crate::details::encodings::LinearSrgbA;
    #[doc(inline)]
    pub use crate::details::encodings::SrgbAU8;
    #[doc(inline)]
    pub use crate::details::encodings::SrgbU8;
}

#[doc(inline)]
pub use color::Color;

#[doc(inline)]
pub use traits::ColorEncoding;

#[doc(inline)]
pub use traits::WorkingEncoding;

#[doc(inline)]
pub use traits::PerceptualEncoding;

/// Like [`Into`] but specialized for use with `colstodian` [`Color`] types.
///
/// This trait exists so that functions can accept colors in a variety of encodings
/// generically in an ergonomic fashion. [`ColorInto`] is blanket implemented generically
/// so that, if you have a function parameter `impl ColorInto<Color<SomeEncoding>>`,
/// a [`Color`] in any other encoding that is able to [`.convert::<SomeEncoding>()`][Color::convert]
/// can be passed into that function as argument directly.
///
/// See [the docs of the `convert` method on `Color`][Color::convert] for more.
///
/// # Example
///
/// ```
/// # use colstodian::*;
/// # use colstodian::details::encodings::*;
/// # use colstodian::equals_eps::*;
/// type MyColor = Color<LinearSrgb>;
///
/// fn test_fn(input: impl ColorInto<MyColor>) {
///     let input: MyColor = input.color_into();
///     let correct = Color::linear_srgb(0.14703, 0.42327, 0.22323);
///     assert_eq_eps!(input, correct, 0.00001);
/// }
///
/// test_fn(Color::srgb_u8(107, 174, 130));
/// test_fn(Color::srgb_f32(0.41961, 0.68235, 0.5098));
/// ```
pub trait ColorInto<DstCol> {
    fn color_into(self) -> DstCol;
}

use details::traits::ConvertFrom;
use details::traits::LinearConvertFromRaw;

impl<SrcEnc, DstEnc> ColorInto<Color<DstEnc>> for Color<SrcEnc>
where
    SrcEnc: ColorEncoding,
    DstEnc: ColorEncoding + ConvertFrom<SrcEnc>,
    DstEnc::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
{
    #[inline(always)]
    fn color_into(self) -> Color<DstEnc> {
        self.convert()
    }
}

/// Helper for use in tests and doctests
#[doc(hidden)]
pub mod equals_eps {
    use super::*;
    use reprs::*;
    use traits::*;

    pub trait EqualsEps<T> {
        fn eq_eps(self, other: Self, eps: T) -> bool;
    }

    impl EqualsEps<f32> for f32 {
        fn eq_eps(self, other: f32, eps: f32) -> bool {
            (self - other).abs() <= eps
        }
    }

    impl EqualsEps<u8> for u8 {
        fn eq_eps(self, other: u8, eps: u8) -> bool {
            (self as i32 - other as i32).abs() as u8 <= eps
        }
    }

    impl EqualsEps<u8> for U8Repr {
        fn eq_eps(self, other: U8Repr, eps: u8) -> bool {
            self[0].eq_eps(other[0], eps)
                && self[1].eq_eps(other[1], eps)
                && self[2].eq_eps(other[2], eps)
        }
    }

    impl EqualsEps<u8> for U8ARepr {
        fn eq_eps(self, other: U8ARepr, eps: u8) -> bool {
            self[0].eq_eps(other[0], eps)
                && self[1].eq_eps(other[1], eps)
                && self[2].eq_eps(other[2], eps)
                && self[3].eq_eps(other[3], eps)
        }
    }

    impl EqualsEps<f32> for F32Repr {
        fn eq_eps(self, other: F32Repr, eps: f32) -> bool {
            self[0].eq_eps(other[0], eps)
                && self[1].eq_eps(other[1], eps)
                && self[2].eq_eps(other[2], eps)
        }
    }

    impl EqualsEps<f32> for F32ARepr {
        fn eq_eps(self, other: F32ARepr, eps: f32) -> bool {
            self[0].eq_eps(other[0], eps)
                && self[1].eq_eps(other[1], eps)
                && self[2].eq_eps(other[2], eps)
                && self[3].eq_eps(other[3], eps)
        }
    }

    impl<E: ColorEncoding> EqualsEps<<E::Repr as ColorRepr>::Element> for Color<E>
    where
        E::Repr: EqualsEps<<E::Repr as ColorRepr>::Element>,
    {
        fn eq_eps(self, other: Color<E>, eps: <E::Repr as ColorRepr>::Element) -> bool {
            self.repr.eq_eps(other.repr, eps)
        }
    }

    #[macro_export]
    macro_rules! assert_eq_eps {
        ($left:expr, $right:expr, $eps:expr) => {{
            match (&($left), &($right)) {
                (left_val, right_val) => {
                    if !(left_val.eq_eps(*right_val, $eps)) {
                        // The reborrows below are intentional. Without them, the stack slot for the
                        // borrow is initialized even before the values are compared, leading to a
                        // noticeable slow down.
                        panic!(
                            r#"assertion failed: `(left ~= right with epsilon {})`
    left: `{:?}`,
    right: `{:?}`"#,
                            $eps, &*left_val, &*right_val
                        )
                    }
                }
            }
        }};
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use encodings::*;
    use equals_eps::*;
    use glam::Vec3;

    #[test]
    fn basic() {
        let grey_f32 = Color::srgb_f32(0.5, 0.5, 0.5);
        let grey_u8 = Color::srgb_u8(127, 127, 127);

        assert_eq_eps!(grey_f32.convert::<SrgbU8>(), grey_u8, 0);

        let col = Color::srgb_u8(102, 51, 153);
        let correct = Color::linear_srgb(0.13287, 0.0331, 0.31855);

        assert_eq_eps!(col.convert::<LinearSrgb>(), correct, 0.0001);
    }

    #[test]
    fn deref() {
        let col: Color<SrgbF32> = Color::srgb_f32(0.2, 0.3, 0.4);
        let r = col.r;
        let g = col.g;
        let b = col.b;

        assert_eq_eps!(r, 0.2, 0.0001);
        assert_eq_eps!(g, 0.3, 0.0001);
        assert_eq_eps!(b, 0.4, 0.0001);
    }

    #[test]
    fn deref_alpha() {
        let col: Color<SrgbAF32> = Color::srgba_f32(0.2, 0.3, 0.4, 0.5);
        let r = col.r;
        let g = col.g;
        let b = col.b;
        let alpha = col.a;

        assert_eq_eps!(r, 0.2, 0.0001);
        assert_eq_eps!(g, 0.3, 0.0001);
        assert_eq_eps!(b, 0.4, 0.0001);
        assert_eq_eps!(alpha, 0.5, 0.0001);
    }

    #[test]
    fn color_into_trait() {
        type MyColorTy = Color<LinearSrgb>;
        fn test_fn(input: impl ColorInto<MyColorTy>) {
            let input: MyColorTy = input.color_into();
            let correct = Color::linear_srgb(0.14703, 0.42327, 0.22323);
            assert_eq_eps!(input, correct, 0.0001);
        }

        test_fn(Color::srgb_u8(107, 174, 130));
        test_fn(Color::srgb_f32(0.41961, 0.68235, 0.5098));
    }

    #[test]
    fn working_space_math() {
        let col = Color::linear_srgb(1.0, 1.0, 1.0);

        let mut col = col * 0.5;
        assert_eq_eps!(col, Color::linear_srgb(0.5, 0.5, 0.5), 0.00001);

        col *= Vec3::new(0.5, 2.0, 0.2);
        assert_eq_eps!(col, Color::linear_srgb(0.25, 1.0, 0.1), 0.00001);

        let mut col2 = Color::linear_srgb(1.0, 1.0, 1.0) + col;
        assert_eq_eps!(col2, Color::linear_srgb(1.25, 2.0, 1.1), 0.00001);

        col2 -= Color::linear_srgb(0.25, 1.0, 0.1);
        assert_eq_eps!(col2, Color::linear_srgb(1.0, 1.0, 1.0), 0.00001);

        col2 /= Vec3::new(2.0, 2.0, 2.0);
        assert_eq_eps!(col2, Color::linear_srgb(0.5, 0.5, 0.5), 0.00001);

        col2 = col2 / 0.1;
        assert_eq_eps!(col2, Color::linear_srgb(5.0, 5.0, 5.0), 0.00001);
    }

    #[test]
    fn perceptual_blend() {
        let start = Color::srgb_u8(105, 220, 58);
        let end = Color::srgb_u8(10, 20, 100);

        let blend_oklab = start
            .convert::<Oklab>()
            .perceptual_blend(end.convert(), 0.5);

        let blend = blend_oklab.convert::<SrgbU8>();

        assert_eq_eps!(
            blend_oklab,
            Color::oklab(0.52740586, -0.085545816, 0.004893869),
            0.0001
        );
        assert_eq_eps!(blend, Color::srgb_u8(35, 123, 105), 0);
    }
}
