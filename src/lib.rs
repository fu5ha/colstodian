//! An opinionated color management library built on top of [`kolor`](https://docs.rs/kolor).
//!
//! # Introduction
//!
//! `colstodian` is a practical color management library for games and graphics.
//! It encodes various information about a color either statically
//! in the Rust type system (as with the strongly-typed [`Color`]),
//! or as data contained in the type (as with the dynamically-typed [`DynamicColor`]).
//!
//! Although it is designed to prevent footguns wherever possible, in order to make use of this
//! library, you should have a well-working understanding of basic color science and encoding
//! principles. As such, I highly recommend you give sections 2.0 and 2.1 [of this document](#res1)
//! a read, as it is one of the main inspirations for how this library is structured:
//!
//! All colors in Ark will now be accompanied either statically or dynamically with two important
//! pieces of metadata, a **color space** and a **state**. A basic background on color encoding is
//! necessary to understand what these pieces of metadata are and why they are important.
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
//! ways, and the end result of what those components mean is very different. There are two
//! important pieces of metadata about a color which inform how we are meant to interpret its
//! component values: the color's **Color Space** and its **State**.
//!
//! ### Color Spaces
//!
//! A "color space" is a fairly nebulous term which has different definitions depending on who you
//! talk to, but the basic idea is that it provides a specific organization of color data in an
//! agreed-upon format. The color space provides almost all the information needed to fully
//! interpret the component data. However, it is missing one important piece of metadata which is
//! relevant when working with rendered scenes that may have higher dynamic range within the scene
//! than an actual display is capable of displaying (a computer monitor cannot replicate the
//! brightness of the sun, but within the renderer, we want to actually simulate those high
//! brightnesses). That is where the color state comes in.  
//!
//! ### Color State
//!
//! As we have discussed, all colors have units. Sometimes a color’s units are explicit, such as
//! measuring the emitted light from a display using a radiometric measurement tool and being able
//! to reference pixel values in a color space built for that. Other times, the units are only
//! indirectly related to the real world, but come with a mathematical conversion to measurable
//! quantities. For example, in the case of display technology, common color encodings include
//! sRGB, DCI-P3, and BT.2020, which are all standards which actual monitors attempt to replicate.
//!
//! However, considering color as a displayed quantity only provides part of the color encoding
//! story. In addition to relating color values to **display measurements**, as we did above, one
//! can also relate color values to the performance characteristics of an **input device** (i.e., a
//! camera, or in our case, a virtual camera in a 3d renderer). In this case, we are quantifying
//! color values which originated in the (virtual) **scene**, rather than ones being displayed on a
//! display. This kind of color can be measured in real world units as well. In the case of a 3d
//! renderer, these units are often defined in the renderer as a photometric quantity like
//! luminance, with the relation to reference color values dictated by a defined transformation.
//!
//! It is a meaningful abstraction to categorize colors based on this distinction of *input* versus
//! *output* reference. We refer to this difference as a color's **State**. Colors which are
//! defined in relation to *display characteristic* are called **Display-referred**, while color
//! spaces which are defined in relation to *input devices* (scenes) are **Scene-referred**.
//!
//! # Overview
//!
//! `colstodian` is broken up into two 'halves', a statically-typed half which is meant to be
//! used as much as possible to help you prevent errors at compile time through leveraging the
//! Rust type system, and a dynamically-typed half which is meant to be used when serializing
//! and deserializing colors and otherwise interacting with colors from dynamic sources not
//! known at compile time.
//!
//! The core of the statically-typed half is the [`Color`] type, which encodes two important pieces
//! of metadata about the color in its type signature (`Color<Space, State>`): the color's **color
//! space** and **state**. If you read the color encoding basics above (you did, didn't you? ;) )
//! then you should have a decent idea of what both of these things mean. To be clear, the **color
//! space** encodes the **primaries**, **white point**, and **transfer functions** upon which the
//! color values are based. The **state** encodes in which "direction" we relate the color values
//! to real-world quantities: either **scene-referred** or **display-referred**. Types which
//! implement the [`ColorSpace`] and [`State`] traits encode this information statically. Color
//! spaces can be found in the [`spaces`] module and states can be found in the [`states`] module.
//!
//! The core of the dynamically-typed half is the [`DynamicColor`] type, which encodes the color
//! space and state as data stored in the type at runtime. It stores these as
//! [`DynamicColorSpace`]s and [`DynamicState`]s.
//!
//! # Example
//!
//! Let's say we have a color that we got from an asset loaded from a color image or a color
//! picker, which are often in the encoded sRGB color space.
//!
//! ```rust
//! # use colstodian::*;
//! let loaded_asset_color = color::srgb_u8(128, 128, 128).to_f32();
//! ```
//!
//! But wait, we can't do much with this color yet...
//!
//! ```rust
//! # use colstodian::*;
//! # let loaded_asset_color = color::linear_srgb::<Display>(0.5, 0.5, 0.5); // fake it so that doc tests pass
//! let my_other_color = loaded_asset_color * 5.0; // oops, compile error!
//! ```
//!
//! This color is encoded in a non-linear format. You can think of this much like as if a file was
//! compressed as a ZIP. Doing operations directly on the zipped bytes is nonsensical. First we
//! need to decode it to work on the raw data. In the same way, before we can do math on this
//! color, we need to convert it to a working color space.  
//!
//! Encoded color spaces all have a working color space that they can decode to directly. This
//! will be the least expensive and most natural conversion if you want to work with them directly.
//! For example, an [EncodedSrgb] color will decode to a [LinearSrgb] color:
//!
//! ```rust
//! # use colstodian::*;
//! # let loaded_asset_color = color::srgb(0.5, 0.5, 0.5);
//! // Note the type annotation here is unnecessary, but is useful for illustrative purposes.
//! let decoded: Color<LinearSrgb, Display> = loaded_asset_color.decode();
//!
//! let my_other_color = decoded * 0.5; // yay, it works!
//! ```
//!
//! You can also convert an encoded color fully to a specific working space if you have one
//! in mind. For example, if you want to blend between two colors, you might convert them to
//! the [Oklab] color space:
//!
//! ```rust
//! # use colstodian::*;
//!
//! let oklab1 = color::srgb_u8(128, 12, 57).to_f32().convert::<Oklab>();
//! let oklab2 = color::srgb_u8(25, 35, 68).to_f32().convert::<Oklab>();
//!
//! let blended = oklab1.blend(oklab2, 0.5); // Blend half way between the two colors
//! ```
//!
//! This is also the first time we see the [`convert`][Color::convert] method, which we'll be
//! using, along with its sibling [`convert_to`][Color::convert_to], to do most of our conversions.
//! You can use this to do pretty much any conversion you like, so long as you stay within the same
//! [State]. See the docs of that method for more information. Generally, you'll want to convert
//! the color to some output color space before actually using it. It's quite common to use
//! [EncodedSrgb] for this purpose. This is also quite simple with `convert`:
//!
//! ```rust
//! # use colstodian::*;
//! # let blended: Color<Oklab, Display> = Color::new(1.0, 1.0, 1.0);
//! // Note the slightly different style. Here we annotate the type of `output`
//! // rather than using the turbofish operator to specify the destination color
//! // space, and Rust infers the type on the `convert` method for us.
//! let output: Color<EncodedSrgb, Display> = blended.convert();
//!
//! // Some applications will want a color in the form of an array of `u8`s or a packed `u32`.
//! // Certain encoded color spaces will allow you to convert a color in that
//! // space to/from a 8-bit encoded form. EncodedSrgb is one of those:
//! let output_u8: ColorU8<_, _> = output.to_u8();
//!
//! // From here we can either get the raw array of `u8`s, or if it was a
//! // `ColorAlphaU8`, we could convert it to a packed `u32`.
//! let output_u8_array: [u8; 3] = output_u8.raw;
//! ```
//!
//! Here we can see an example of where [`convert_to`][Color::convert_to] may be preferrable over
//! [`convert`][Color::convert]. Notice how we are using the type [`Color<EncodedSrgb, Display>`]
//! quite often? You might want to create a type alias for this type called, for example, `Asset`.
//! Wouldn't it be convenient to also be able to `convert` to a type alias of [`Color`]? Well, with
//! [`convert_to`][Color::convert_to], you can! This is particularly useful when you don't want to
//! bind the output to a variable directly, so you can't take advantage of type inference and
//! need to use the turbofish operator. For example, let's rewrite the previous blending example:
//!
//! ```rust
//! # use colstodian::*;
//! // You could have these defined and used throughout your codebase.
//! type Perceptual = Color<Oklab, Display>;
//! type Srgb = Color<EncodedSrgb, Display>;
//!
//! let color_1 = color::srgb_u8(128, 12, 57).to_f32();
//! let color_2 = color::srgb_u8(25, 35, 68).to_f32();
//!
//! let blended_u8 = color_1.convert_to::<Perceptual>().blend(
//!     color_2.convert_to::<Perceptual>(),
//!     0.5
//! ).convert_to::<Srgb>().to_u8();
//! ```
//!
//! [`convert_to`][Color::convert_to] can also take a [`ColorSpace`] as a Query directly. However,
//! because it's more generic than [`convert`][Color::convert], Rust's type system will often not
//! be able to infer the type of Query, for example annotating a type as being a specific [`Color`]
//! type and then calling `other_color.convert_to()` will give you a type annotation needed error.
//!
//! Going back to what you can and cannot do with different [`Color`] types,
//! note that you can break out of the restrictions imposed by the type system
//! or otherwise get the raw color by accessing `color.raw`:
//!
//! ```rust
//! # use colstodian::*;
//! let mut encoded_color = color::srgb_u8(127, 127, 127).to_f32();
//! encoded_color.raw *= 0.5; // This works! But be careful that you know what you're doing.
//! ```
//!
//! You can also access the components of a color by that component's name. For example,
//! a [Linear sRGB][LinearSrgb] color has components `r`, `g`, and `b`, so you can do:
//!
//! ```rust
//! # use colstodian::*;
//! # let linear_srgb_color = color::linear_srgb::<Display>(0.5, 0.5, 0.5);
//! let red_component = linear_srgb_color.r;
//! ```
//!
//! However, if a color is in a different color space, for example [`ICtCpPQ`], which has different
//! component names, then you would access those components accordingly:
//!
//! ```rust
//! # use colstodian::*;
//! let col: Color<ICtCpPQ, Display> = Color::new(1.0, 0.2, 0.2);
//!
//! let intensity = col.i; // access I (Intensity) component through .i
//! let ct = col.ct; // access Ct (Chroma-Tritan) component through .ct
//! let cp = col.cp; // access Cp (Chroma-Protan) component through .cp
//! ```
//!
//! One more quite useful tool is the [`ColorInto`] trait. [`ColorInto`] is
//! a trait meant to be used as a replacement for [`Into`] in situations where you want
//! to bound a type as being able to be converted into a specific type of color. You can
//! call [`.into`][ColorInto::into] on a type that implements [`ColorInto<T>`]
//! and you will get a `T`.
//!
//! This example snippet puts together much of what we've learned so far.
//!
//! ```rust
//! # use colstodian::*;
//! fn tint_color(input_color: impl ColorInto<Color<AcesCg, Display>>) -> Color<AcesCg, Display> {
//!     let color = input_color.into();
//!     let tint: Color<AcesCg, Display> = Color::new(0.5, 0.8, 0.4);
//!     color * tint
//! }
//!
//! let color = color::srgb_u8(225, 200, 86).to_f32();
//! let tinted: Color<EncodedSrgb, Display> = tint_color(color).convert();
//!
//! println!("Pre-tint: {}, Post-tint: {}", color, tinted);
//! ```
//!
//! Now, let's go back to our previous `decoded` color from the begining.
//!
//! Let's say that instead of blending perceptually between colors, we are creating a 3d rendering
//! engine. In this case, we probably want to do the actual shading math in a color space with a
//! wider (sharper) gamut (the reasons for this are outside the scope of this demo). The
//! [ACEScg][AcesCg] space is ideal for this.
//!
//! Since both color spaces are linear, the ideally optimized transformation is a simple
//! 3x3 matrix * 3 component vector multiplication. `colstodian` is architected such that we can
//! still just use the [`convert`][Color::convert] method to convert between these spaces and it
//! will indeed optimize fully down to just that multiplication.
//!
//! ```rust
//! # use colstodian::*;
//! # let decoded = color::linear_srgb::<Display>(0.5, 0.5, 0.5);
//! let col: Color<AcesCg, Display> = decoded.convert();
//! ```
//!
//! Now, we come to a bit of a subtle operation. Here we will convert the color from being in a
//! display-reffered state to being in a scene-referred state. This operation is not necessarily
//! concrete, and is dependent on the thing you are converting. Going from display-referred to
//! scene-referred, we are converting from a bounded dynamic range with physical reference units
//! for the color component values being (with a properly calibrated monitor) the **display
//! standard specification**, to an unbounded dynamic range, with color components in the range
//! `[0..inf)` and the physical reference units for these component values being the units used in
//! the **scene,** which are defined by the renderer itself. In most cases, these units will be in
//! a measurement of photometric luminance like
//! [Cd/m^2 aka nits](https://en.wikipedia.org/wiki/Candela_per_square_metre).
//!
//! One possible use of this conversion is the case of an emissive texture, where we may want to
//! modify the bounded [illuminance (i.e. lux)](https://en.wikipedia.org/wiki/Illuminance) of the
//! color we stored in the texture by some unbounded *power* value stored elsewhere. In this way,
//! we can make emissive materials just as powerful as any other light in the scene.
//!
//! ```rust
//! # use colstodian::*;
//! # let col = color::acescg::<Display>(0.5, 0.5, 0.5);
//! let power = 5.0; // Say you loaded this from an asset somewhere
//!
//! // Note the `Scene` state... previously, all colors have been in `Display` state.
//! let emissive_col: Color<AcesCg, Scene> = col.convert_state(|c| c * power);
//! ```
//!
//! Now we can do the actual rendering math, using this scene-referred color value.
//!
//! ```text
//! // ... rendering math here ...
//! ```
//!
//! Okay, so let's say we've ended up with a final color for a pixel, which is still scene-referred
//! in the ACEScg color space, representing the luminance reaching the camera from a specific
//! direction (namely, the direction corresponding to the pixel we are shading).
//!
//! ```rust
//! # use colstodian::*;
//! let rendered_col = color::acescg::<Scene>(5.0, 4.0, 4.5); // let's just say this is the computed final color.
//! ```
//!
//! Now we need to do the opposite of what we did before and map the infinite dynamic range of a
//! scene-referred color outputted by the renderer to the finite dynamic range which can be
//! displayed on a display. For an output display which is "SDR" (i.e. not an HDR-enabled TV or
//! monitor), a fairly aggressive S-curve style tonemap is a good option. We provide a couple of
//! options in the [`tonemap`] module.
//!
//! ```rust
//! # use colstodian::*;
//! # let rendered_col = color::acescg::<Scene>(5.0, 4.0, 4.5);
//! use tonemap::{Tonemapper, PerceptualTonemapper, PerceptualTonemapperParams};
//!
//! // In theory you could change the parameters to taste here.
//! let params = PerceptualTonemapperParams::default();
//! let tonemapped: Color<AcesCg, Display> = PerceptualTonemapper::tonemap(rendered_col, params).convert();
//! ```
//!
//! Now, our color is display-referred within a finite (`[0..1]`) dynamic range. However, we
//! haven't chosen an actual specific display to encode it for. This is what the sRGB standard can
//! help with, which is most likely the standard upon which an LDR monitor will be based. We can
//! convert our color to [encoded sRGB][EncodedSrgb] just like we showed before.
//!
//! ```rust
//! # use colstodian::*;
//! # let tonemapped = color::acescg::<Display>(5.0, 4.0, 4.5);
//! let encoded = tonemapped.convert::<EncodedSrgb>(); // Ready to display or write to an image.
//!
//! // Again, if your output format needs `u8`s (say, an 8-bit PNG image), you can use the `to_u8()` method.
//! let u8s: [u8; 3] = encoded.to_u8().raw;
//! ```
//!
//! Alternatively, we could output to a different display, for example to a wide-gamut but still
//! LDR BT.2020 calibrated display:
//!
//! ```rust
//! # use colstodian::*;
//! # let tonemapped = color::acescg::<Display>(5.0, 4.0, 4.5);
//! let encoded = tonemapped.convert::<EncodedBt2020>();
//! ```
//!
//! This doesn't cover displaying to an HDR display yet, nor the use of colors with an alpha
//! channel, but it soon will!
//!
//! # Further Resources
//!
//! Here is a curated list of further resources to check out for information about color encoding
//! and management:
//!
//! 1. <span id="res1">An overview of color management from a cinematic perspective</span>
//!    (HIGHLY recommended sections 2.0 and 2.1):<br/>
//!    <http://github.com/jeremyselan/cinematiccolor/raw/master/ves/Cinematic_Color_VES.pdf>
//! 2. The Hitchhiker's Guide to Digital Color:<br/> <https://hg2dc.com/>
//! 3. Alex Fry (DICE/Frostbite) on HDR color management in Frostbite:<br/>
//!    <https://www.youtube.com/watch?v=7z_EIjNG0pQ>
//! 4. Timothy Lottes (AMD) on "variable" dynamic range color management:<br/>
//!    <https://www.gdcvault.com/play/1023512/Advanced-Graphics-Techniques-Tutorial-Day>
//! 5. Hajime Uchimura and Kentaro Suzuki on HDR and Wide color strategies in Gran Turismo SPORT:
//!    <br/> <https://www.polyphony.co.jp/publications/sa2018/>
#![cfg_attr(not(feature = "std"), no_std)]

pub use kolor;

/// Types representing different color spaces.
///
/// For more information, see the documentation for the corresponding dynamic color space (you can figure out the
/// corresponding dynamic color space by looking at the implementation of the [`ColorSpace`] trait on a specific
/// color space struct).
#[rustfmt::skip]
pub mod spaces;
#[doc(inline)]
pub use spaces::DynamicColorSpace;
pub use spaces::*;

/// Contains types relating to a color's state.
pub mod states;

#[doc(inline)]
pub use states::{Display, DynamicState, Scene};

/// Contains types relating to a color's alpha state.
pub mod alpha_states;

#[doc(inline)]
pub use alpha_states::{DynamicAlphaState, Premultiplied, Separate};

/// Contains tonemappers, useful for mapping scene-referred HDR values into display-referred values
/// within the concrete dynamic range of a specific display.
pub mod tonemap;

pub mod component_structs;
pub use component_structs::*;

/// Contains color types and helper functions.
#[macro_use]
pub mod color;

#[doc(inline)]
pub use color::{Color, ColorAlpha};

#[cfg(not(target_arch = "spirv"))]
pub use color::{ColorU8, ColorU8Alpha};

#[cfg(not(target_arch = "spirv"))]
#[doc(inline)]
pub use color::{DynamicColor, DynamicColorAlpha};

/// The traits which form the backbone of the strongly-typed [`Color`] & [`ColorAlpha`].
pub mod traits;

#[doc(inline)]
pub use traits::{AlphaState, ColorInto, ColorSpace, State};

/// Error handling types.
#[cfg(not(target_arch = "spirv"))]
pub mod error;

#[doc(inline)]
#[cfg(not(target_arch = "spirv"))]
pub use error::{ColorError, ColorResult};

#[cfg(test)]
mod tests {
    use super::*;

    trait EqualsEps {
        fn eq_eps(self, other: Self, eps: f32) -> bool;
    }

    impl EqualsEps for f32 {
        fn eq_eps(self, other: f32, eps: f32) -> bool {
            (self - other).abs() <= eps
        }
    }

    impl<Spc, St> EqualsEps for Color<Spc, St> {
        fn eq_eps(self, other: Color<Spc, St>, eps: f32) -> bool {
            self.raw.x.eq_eps(other.raw.x, eps)
                && self.raw.y.eq_eps(other.raw.y, eps)
                && self.raw.z.eq_eps(other.raw.z, eps)
        }
    }

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

    #[test]
    fn basic() {
        let orig = Color::<EncodedSrgb, Display>::new(0.5, 0.5, 0.5);
        let col: Color<LinearSrgb, Display> = orig.convert::<LinearSrgb>();
        let col: Color<AcesCg, Display> = col.convert();
        let oklab = col.convert::<Oklab>();

        assert_eq_eps!(orig.convert::<AcesCg>(), col, 0.0001);
        assert_eq_eps!(orig.convert::<Oklab>(), oklab, 0.0001);
    }

    #[test]
    fn round_trip() {
        let orig = Color::<EncodedSrgb, Display>::new(0.5, 0.5, 0.5);
        let col: Color<LinearSrgb, Display> = orig.convert();
        let col: Color<AcesCg, Display> = col.convert();
        let col: Color<AcesCg, Scene> = col.convert_state(|c| c * 5.0);

        let col: Color<AcesCg, Display> = col.convert_state(|c| c / 5.0);
        let col: Color<LinearSrgb, Display> = col.convert();
        let fin: Color<EncodedSrgb, Display> = col.convert();

        assert_eq_eps!(orig, fin, 0.0001);
    }

    #[test]
    fn deref() {
        let col: Color<EncodedSrgb, Display> = Color::new(0.2, 0.3, 0.4);
        let r = col.r;
        let g = col.g;
        let b = col.b;

        assert_eq_eps!(r, 0.2, 0.0001);
        assert_eq_eps!(g, 0.3, 0.0001);
        assert_eq_eps!(b, 0.4, 0.0001);
    }

    #[test]
    fn deref_alpha() {
        let colalpha: ColorAlpha<EncodedSrgb, Display, Premultiplied> =
            ColorAlpha::new(0.2, 0.3, 0.4, 0.5);
        let r = colalpha.col.r;
        let g = colalpha.col.g;
        let b = colalpha.col.b;
        let alpha = colalpha.alpha;

        assert_eq_eps!(r, 0.2, 0.0001);
        assert_eq_eps!(g, 0.3, 0.0001);
        assert_eq_eps!(b, 0.4, 0.0001);
        assert_eq_eps!(alpha, 0.5, 0.0001);
    }
}
