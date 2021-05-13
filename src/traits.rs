//! [`Color`] system.

use super::*;

/// A type that implements ColorSpace represents a specific color space. See the documentation
/// of [`DynamicColorSpace`] for more information about what a color space is.
pub trait ColorSpace {
    const SPACE: DynamicColorSpace;
}

/// Marks a type as representing a linear color space.
pub trait LinearColorSpace {}

/// A type that implements this trait is a color space for which a linear conversion
/// from `SrcSpace` to `Self` exists.
pub trait LinearConvertFrom<SrcSpc> {
    // TODO: use const Mat3s instead
    const MATRIX: [f32; 9];
}

/// A type that implements this trait is a color space for which a single nonlinear
/// transform function exists to decode a color from `SrcSpace` into `Self`. For example,
/// [`LinearSrgb`] implements [`DecodeFrom<EncodedSrgb>`].
pub trait DecodeFrom<SrcSpc> {
    /// Decode the raw color from `SrcSpace` into the space represented by `Self`
    fn decode_raw(color: Vec3) -> Vec3;
}

/// A type that implements this trait is a color space for which a single nonlinear
/// transform function exists to encode a color from `SrcSpace` into `Self`. For example,
/// [`EncodedSrgb`] implements [`EncodeFrom<LinearSrgb>`].
pub trait EncodeFrom<SrcSpace> {
    /// Encode the raw color from `SrcSpace` into the space represented by `Self`
    fn encode_raw(color: Vec3) -> Vec3;
}

/// A type that implements this trait can be converted directly to and from
/// an appropriately sized array of `u8`s.
pub trait AsU8Array {}

/// A type that implements this trait represents a color's State.
///
/// All colors have units. Sometimes a color's units are explicit, such as measuring the emitted
/// light from a display using a spectroradiometer and being able to reference pixel values in CIE XYZ cd/m2.
/// Other times, the units are only indirectly related to the real world, and then providing a
/// mathematical conversion to measurable quantities. For example, in the case of display technology, common color encodings
/// (relations of code value to measurable XYZ performance) include sRGB, DCI-P3, and BT.2020.
///
/// Howver, considering color as a displayed quantity only provides part of the color encoding story. In addition to relating RGB
/// values to display measurements, one can also relate RGB values to the performance characteristics of an
/// *input device* (i.e., a camera, or a virtual camera in a 3d renderer). Input colorimetry can be measured in real world units as well.
/// In the case of a 3d renderer, these units are often (or at least should be) defined in the renderer as a radiometric quantity like
/// radiance, with the relation to XYZ values dictated by a linear transformation to the rendering color space.
/// Even in the case of a real world camera, it is not difficult to measure an input spectra with the spectrophotometer
/// in XYZ, and then compare this to the RGB values output from the camera.
///
/// It is a meaningful abstraction to categorize color spaces by the “direction” of this relationship to real world
/// quantities, which we refer to as State. Colors which are defined in relation to display
/// characteristic are called [`Display`]-referred, while color spaces which are defined in relation to input
/// devices (scenes) are [`Scene`]-referred.
pub trait State {
    const STATE: DynamicState;
}

/// A type that implements this trait represents a color's alpha state.
///
/// A color can either have a [`Separate`] alpha channel or have been pre-multiplied
/// with its alpha channel and so have [`Premultiplied`] alpha.
pub trait AlphaState {
    const STATE: DynamicAlphaState;
}
