//! An opinionated color management library built on top of [`kolor`](https://docs.rs/kolor).
//!
//! # Introduction
//!
//! `colstodian` is a practical color management library for games and graphics.
//! It encodes various information about a color either statically
//! in the Rust type system (as with the strongly-typed [`Color`]),
//! or as data contained in the type (as with the dynamically-typed [`DynamicColor`]).
//!
//! Although it is designed to prevent footguns wherever possible, in order to make use of this library,
//! you should have a well-working understanding of basic color science and encoding principles.
//! As such, I highly recommend you give sections 2.0 and 2.1 of this document a read, as it is one
//! of the main inspirations for how this library is structured:
//!
//! ## Required Reading
//!
//! * <http://github.com/jeremyselan/cinematiccolor/raw/master/ves/Cinematic_Color_VES.pdf>
//!
//! # Overview
//!
//! `colstodian` is broken up into two 'halves', a statically-typed half which is meant to be
//! used as much as possible to help you prevent errors at compile time through leveraging the
//! Rust type system, and a dynamically-typed half which is meant to be used when serializing
//! and deserializing colors and otherwise interacting with colors from dynamic sources not
//! known at compile time.
//!
//! The core of the statically-typed half is the [`Color`] type, which encodes
//! two important pieces of metadata about the color in its type signature (`Color<Space, State>`): the
//! color's **color space** and **state**. If you read the required reading (you did, didn't you? ;))
//! then you should have a decent idea of what both of these things mean. To be clear, the **color space**
//! encodes the **primaries**, **white point**, and **transfer functions** upon which the color values are
//! based. The **state** encodes in which "direction" we relate the color values to real-world quantities:
//! either **scene-referred** or **display-referred**. Types which implement the [`ColorSpace`] and
//! [`State`] traits encode this information statically. Color spaces can be found in the `spaces` module
//! and states can be found in the `states` module.
//!
//! The core of the dynamically-typed half is the [`DynamicColor`] type, which encodes the color space
//! and state as data stored in the type at runtime. It stores these as [`DynamicColorSpace`]s and [`DynamicState`]s.
//!
//! # Example
//!
//! Let's say we have a color that we got from an asset loaded from a color image or a color picker,
//! which are often in the encoded sRGB color space.
//! ```rust
//! # use colstodian::*;
//! let loaded_asset_color = color::srgb(0.5, 0.5, 0.5);
//! ```
//!
//! But wait, we can't do much with this color yet...
//! ```
//! # use colstodian::*;
//! # let loaded_asset_color = color::linear_srgb::<Display>(0.5, 0.5, 0.5);
//! let my_other_color = loaded_asset_color * 5.0; // oops, compile error!
//! ```
//!
//! This color is encoded in a non-linear format. You can think of this much like
//! as if a file was compressed as a ZIP. Doing operationd directly on the zipped
//! bytes is nonsensical. First we need to decode it to work on the raw data.
//! In the same way, before we can do math on this color, we need to convert it to a linear color space.
//! In this case, the best way to do that is by decoding it to Linear sRGB.
//! ```rust
//! # use colstodian::*;
//! # let loaded_asset_color = color::srgb(0.5, 0.5, 0.5);
//! let decoded = loaded_asset_color.decode::<LinearSrgb>();
//!
//! let my_other_color = decoded * 5.0; // yay, it works!
//! ```
//!
//! Also note that you can break out of the restrictions imposed by the type system
//! or otherwise get the raw color by accessing `color.raw`:
//! ```rust
//! # use colstodian::*;
//! let mut loaded_asset_color = color::srgb(0.5, 0.5, 0.5);
//! loaded_asset_color.raw *= 5.0; // This works!
//! ```
//!
//! You can also access the components of a color by that component's name, for example
//! a linear sRGB color has components `r`, `g`, and `b`, so you can do:
//! ```rust
//! # use colstodian::*;
//! # let decoded = color::linear_srgb::<Display>(0.5, 0.5, 0.5);
//! let red_component = decoded.r;
//! ```
//!
//! However, if a color is in a different color space, for example [`ICtCpPQ`], which has different
//! component names, then you would access those components accordingly:
//! ```rust
//! # use colstodian::*;
//! let col: Color<ICtCpPQ, Display> = Color::new(1.0, 0.2, 0.2);
//!
//! let intensity = col.i; // acces I component through .i
//! let ct = col.ct; // access Ct component through .ct
//! ```
//!
//! Now, let's go back to our previous decoded color from the begining.
//!
//! We can now do whatever math we want to this color. Let's say we are creating a 3d rendering
//! engine. In this case, we probably want to do the actual shading math in a color space with a
//! wider (sharper) gamut (the reasons for this are outside the scope of this demo). The ACEScg
//! space is ideal for this.
//!
//! Since both color spaces are linear, we can convert it using convert_linear, which compiles
//! down to a simple 3x3 matrix * 3 component vector multiplication.
//! ```rust
//! # use colstodian::*;
//! # let decoded = color::linear_srgb::<Display>(0.5, 0.5, 0.5);
//! let col: Color<AcesCg, Display> = decoded.convert_linear();
//! ```
//!
//! Now we have a bit of a subtle operation. Here we will convert the color from being in a display-reffered
//! state to being in a scene-referred state. This operation is not necessarily concrete, and is dependent on
//! the thing you are converting. Going from display-referred to scene-referred, we are converting from a
//! bounded dynamic range, with color components in the range `[0..1]` and with the physical reference units for these
//! component values being (with a properly calibrated monitor) the specification of the way those values are
//! *displayed*, to an unbounded dynamic range, with color components in the range `[0..inf)` and the physical reference units
//! for these component values being the units used in the *scene,* defined by the renderer itself.
//!
//! One possible use of this conversion is the case of an emissive texture, where we may want to modify the
//! color we stored in the texture by some *power* value stored elsewhere, so that we can change its intensity
//! like any other light in the scene.
//! ```rust
//! # use colstodian::*;
//! # let col = color::acescg::<Display>(0.5, 0.5, 0.5);
//! let power = 5.0; // Say you loaded this from an asset somewhere
//! let emissive_col: Color<AcesCg, Scene> = col.convert_state(|c| c * power);
//! ```
//!
//! Now we can do the actual rendering math, using this scene-referred color value.
//!
//! ```text
//! // ... rendering math here ...
//! ```
//!
//! Okay, so now we've ended up with a final color for a pixel, which is still a scene-referred color in the
//! ACEScg color space.
//! ```rust
//! # use colstodian::*;
//! let rendered_col = color::acescg::<Scene>(5.0, 4.0, 4.5); // let's just say this is the computed final color.
//! ```
//!
//! Now we need to do the opposite of what we did before, mapping the infinite dynamic range of a
//! scene-referred color outputted by the renderer to a finite dynamic range which can be displayed
//! on a display. For an output display which is "SDR" (i.e. not an HDR-enabled TV or monitor), fairly
//! aggressive s-curve tonemap is a good option. We provide one in the form of the [`LottesTonemapper`],
//! which is a tonemapper inspired by [a talk given by AMD's Timothy Lottes](https://www.gdcvault.com/play/1023512/Advanced-Graphics-Techniques-Tutorial-Day).
//! See the documentation for more information on why it is a good choice.
//! ```rust
//! # use colstodian::*;
//! # let rendered_col = color::acescg::<Scene>(5.0, 4.0, 4.5);
//! let tonemapper = LottesTonemapper::new(Default::default()); // In reality you would customize the parameters here
//! let tonemapped: Color<AcesCg, Display> = rendered_col.tonemap(tonemapper);
//! ```
//!
//! Now our color is display-referred within a finite (`[0..1]`) dynamic range, however we haven't chosen
//! an actual specific display to encode it for. This is what the sRGB standard can help with, which
//! is most likely the standard by upon an LDR monitor will be based. We can convert our color first to
//! linear sRGB and then encode it to sRGB, ready to send directly to the monitor, or to save to a file
//! and be displayed on the web.
//! ```rust
//! # use colstodian::*;
//! # let tonemapped = color::acescg::<Display>(5.0, 4.0, 4.5);
//! let lin_srgb: Color<LinearSrgb, Display> = tonemapped.convert_linear(); // Display-referred to an sRGB display
//! let encoded = lin_srgb.encode::<EncodedSrgb>(); // Encode with sRGB OETF, ready to save to image/send to monitor
//! ```
//!
//! Alternatively, we could output to a different display, for example to a wide-gamut but still LDR
//! BT.2020 calibrated display:
//! ```rust
//! # use colstodian::*;
//! # let tonemapped = color::acescg::<Display>(5.0, 4.0, 4.5);
//! let lin_bt2020: Color<Bt2020, Display> = tonemapped.convert_linear();
//! let encoded = lin_bt2020.encode::<EncodedBt2020>();
//! ```
//!
//! This doesn't cover displaying to an HDR display yet, but it soon will!
//!
//! # Further Resources
//!
//! Here is a curated list of further resources to check out for information about color encoding and management.
//!
//! * The Hitchhiker's Guide to Digital Color: <https://hg2dc.com/>
//! * Alex Fry (DICE/Frostbite) on HDR color management in Frostbite: <https://www.youtube.com/watch?v=7z_EIjNG0pQ>
//! * Timothy Lottes (AMD) on "variable" dynamic range color management: <https://www.gdcvault.com/play/1023512/Advanced-Graphics-Techniques-Tutorial-Day>
//! * Hajime Uchimura and Kentaro Suzuki on HDR and Wide color strategies in Gran Turismo SPORT: <https://www.polyphony.co.jp/publications/sa2018/>

pub use kolor;
pub use glam;
pub(crate) use glam::{Vec3, Vec4, Mat3};

use std::{marker::PhantomData, ops::*};

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use derivative::*;

/// Types representing different color spaces.
///
/// For more information, see the documentation for the corresponding
/// dynamic color space (you can figure out the corresponding dynamic color space by looking at the)
/// implementation of the [`ColorSpace`] trait on a specific color space struct.
pub mod spaces;
pub use spaces::*;

/// Contains types relating to a color's state.
pub mod states;
pub use states::*;

/// Contains types relating to a color's alpha state.
pub mod alpha_states;
pub use alpha_states::*;

/// Contains tonemappers, useful for mapping scene-referred HDR values into display-referred values
/// within the concrete dynamic range of a specific display.
pub mod tonemapper;
pub use tonemapper::*;

pub mod component_structs;

/// Contains the [`Color`] and [`DynamicColor`] types.
pub mod color;
pub use color::*;

/// Contains the [`ColorAlpha`] and [`DynamicColorAlpha`] types.
pub mod color_alpha;
pub use color_alpha::*;

/// The traits which form the backbone of the strongly-typed [`Color`]/[`ColorAlpha`].
pub mod traits;
pub use traits::*;

/// Error handling types.
pub mod error;
pub use error::*;

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
                            r#"assertion failed: `(left ~= right with error {})`
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
    fn round_trip() {
        let orig = Color::<EncodedSrgb, Display>::new(0.5, 0.5, 0.5);
        let col: Color<LinearSrgb, Display> = orig.decode();
        let col: Color<AcesCg, Display> = col.convert_linear();
        let col: Color<AcesCg, Scene> = col.convert_state(|c| c * 5.0);

        let col: Color<AcesCg, Display> = col.convert_state(|c| c / 5.0);
        let col: Color<LinearSrgb, Display> = col.convert_linear();
        let fin: Color<EncodedSrgb, Display> = col.encode();

        assert_eq_eps!(orig, fin, 0.0001);
    }
}
