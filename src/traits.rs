//! [`Color`] system.

use super::*;

use core::fmt;

/// A type that implements ColorSpace represents a specific color space. See the documentation
/// of [`DynamicColorSpace`] for more information about what a color space is.
pub trait ColorSpace: Default + fmt::Display {
    /// The [`DynamicColorSpace`] that this type represents.
    const SPACE: DynamicColorSpace;

    /// The closest linear color space to this space.
    type LinearSpace: LinearColorSpace + ConvertFromRaw<Self>;

    /// The 'bag of components' that this color space uses.
    type ComponentStruct: Clone + Copy + fmt::Display;
}

/// Marks a type as representing a linear color space.
pub trait LinearColorSpace: ColorSpace {}

/// Marks a type as representing a nonlinear color space.
pub trait NonlinearColorSpace: ColorSpace {}

/// Marks a type as representing an encoded color space.
///
/// A color in an [`EncodedColorSpace`] has few operations
/// that can be performed on it other than just converting it
/// to a [`WorkingColorSpace`]
pub trait EncodedColorSpace: ColorSpace {
    type DecodedSpace: WorkingColorSpace + ConvertFromRaw<Self>;
}

/// Marks a type as representing a color space that is not [encoded][EncodedColorSpace] and is therefore
/// able to have many more operations performed on it.
pub trait WorkingColorSpace: ColorSpace {}

/// Performs the raw conversion from the [`ColorSpace`] represented by `SrcSpc` to
/// the [`ColorSpace`] represented by `Self` in three concrete steps, each of which
/// may do some work or be a no-op.
pub trait ConvertFromRaw<SrcSpace: ColorSpace>: ColorSpace {
    fn src_transform_raw(color: Vec3) -> Vec3;
    fn linear_part_raw(color: Vec3) -> Vec3;
    fn dst_transform_raw(color: Vec3) -> Vec3;
}

/// The complement of [`ConvertFromRaw`].
///
/// Performs the raw conversion from the [`ColorSpace`] represented by `Self` to
/// the [`ColorSpace`] represented by `DstSpace` in three concrete steps, each of which
/// may do some work or be a no-op.
///
/// This is automatically implemented for all types that implement [`ConvertFromRaw`],
/// much like how the [From] and [Into] traits work, where [From] gets you [Into] for free.
pub trait ConvertToRaw<DstSpace: ColorSpace>: ColorSpace {
    fn src_transform_raw(color: Vec3) -> Vec3;
    fn linear_part_raw(color: Vec3) -> Vec3;
    fn dst_transform_raw(color: Vec3) -> Vec3;
}

impl<SrcSpace: ColorSpace, DstSpace: ConvertFromRaw<SrcSpace>> ConvertToRaw<DstSpace> for SrcSpace {
    #[inline]
    fn src_transform_raw(color: Vec3) -> Vec3 {
        <DstSpace as ConvertFromRaw<SrcSpace>>::src_transform_raw(color)
    }

    #[inline]
    fn linear_part_raw(color: Vec3) -> Vec3 {
        <DstSpace as ConvertFromRaw<SrcSpace>>::linear_part_raw(color)
    }

    #[inline]
    fn dst_transform_raw(color: Vec3) -> Vec3 {
        <DstSpace as ConvertFromRaw<SrcSpace>>::dst_transform_raw(color)
    }
}

/// A trait meant to be used as a replacement for [`Into`] in situations where you want
/// to bound a type as being able to be converted into a specific type of color.
/// Because of how `colstodian` works and how [`From`]/[`Into`] are implemented, we can't use them directly
/// for this purpose.
///
/// # Example
///
/// ```rust
/// use colstodian::*;
///
/// fn tint_color(input_color: impl ConvertTo<Color<AcesCg, Display>>) -> Color<AcesCg, Display> {
///     let color = input_color.convert();
///     let tint: Color<AcesCg, Display> = Color::new(0.5, 0.8, 0.4);
///     color * tint
/// }
///
/// let color = color::srgb_u8(225, 200, 86);
/// let tinted: Color<EncodedSrgb, Display> = tint_color(color).convert();
///
/// println!("Pre-tint: {}, Post-tint: {}", color, tinted);
/// ```
pub trait ConvertTo<T> {
    fn convert(self) -> T;
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
pub trait State: Default + fmt::Display {
    const STATE: DynamicState;
}

/// A type that implements this trait represents a color's alpha state.
///
/// A color can either have a [`Separate`] alpha channel or have been pre-multiplied
/// with its alpha channel and so have [`Premultiplied`] alpha.
pub trait AlphaState
where
    Self: Default
        + fmt::Display
        + ConvertToAlphaRaw<Separate>
        + ConvertToAlphaRaw<Premultiplied>
        + ConvertFromAlphaRaw<Separate>
        + ConvertFromAlphaRaw<Premultiplied>,
{
    const STATE: DynamicAlphaState;
}

/// Performs the conversion from [alpha state][AlphaState] `SrcAlphaState` into `Self`
/// on a raw color.
pub trait ConvertFromAlphaRaw<SrcAlphaState> {
    fn convert_raw(raw: Vec3, alpha: f32) -> Vec3;
}

impl<T> ConvertFromAlphaRaw<T> for T {
    #[inline]
    fn convert_raw(raw: Vec3, _alpha: f32) -> Vec3 {
        raw
    }
}

/// The complement of [`ConvertFromAlphaRaw`]. Performs the conversion from [alpha state][AlphaState] `Self` into `DstAlphaState`
/// on a raw color.
///
/// This is automatically implemented for all types that implement [`ConvertFromAlphaRaw`],
/// much like how the [From] and [Into] traits work, where [From] gets you [Into] for free.
pub trait ConvertToAlphaRaw<DstAlphaState> {
    fn convert_raw(raw: Vec3, alpha: f32) -> Vec3;
}

impl<SrcAlpha, DstAlpha: ConvertFromAlphaRaw<SrcAlpha>> ConvertToAlphaRaw<DstAlpha> for SrcAlpha {
    fn convert_raw(raw: Vec3, alpha: f32) -> Vec3 {
        <DstAlpha as ConvertFromAlphaRaw<SrcAlpha>>::convert_raw(raw, alpha)
    }
}
