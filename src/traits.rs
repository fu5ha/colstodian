//! [`Color`] system.
use crate::{
    Color, ColorAlpha, ColorResult, DynamicAlphaState, DynamicColor, DynamicColorAlpha,
    DynamicColorSpace, DynamicState, Premultiplied, Separate,
};

use glam::{Vec3, Vec4};

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
/// characteristic are called [`Display`][crate::Display]-referred, while color spaces which are defined in relation to input
/// devices (scenes) are [`Scene`][crate::Scene]-referred.
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

/// A "conversion query" for a [`Color`][crate::Color].
///
/// A type that implements this
/// trait is able to be used as the type parameter for [`Color::convert`][crate::Color::convert].
///
/// The types that implement this trait are:
/// * [`ColorSpace`] types
/// * [`Color`][crate::Color] types (in which case it will be converted to that color's space)
pub trait ColorConversionQuery<SrcSpace: ColorSpace, St: State> {
    type DstSpace: ConvertFromRaw<SrcSpace>;
}

impl<SrcSpace, DstSpace, St> ColorConversionQuery<SrcSpace, St> for Color<DstSpace, St>
where
    SrcSpace: ColorSpace,
    DstSpace: ConvertFromRaw<SrcSpace>,
    St: State,
{
    type DstSpace = DstSpace;
}

/// A "conversion query" for a [`ColorAlpha`][crate::ColorAlpha].
///
/// A type that implements this
/// trait is able to be used as the type parameter for [`ColorAlpha::convert_to`][crate::ColorAlpha::convert_to].
///
/// The types that implement this trait are:
/// * [`ColorSpace`] types
/// * [`AlphaState`] types
/// * [`ColorAlpha`][crate::ColorAlpha] types (in which case it will be converted to that color's space and alpha state)
pub trait ColorAlphaConversionQuery<SrcSpace: ColorSpace, SrcAlpha: AlphaState> {
    type DstSpace: ConvertFromRaw<SrcSpace>;
    type DstAlpha: ConvertFromAlphaRaw<SrcAlpha> + AlphaState;
}

impl<SrcSpace, DstSpc, SrcAlpha, DstAlpha> ColorAlphaConversionQuery<SrcSpace, SrcAlpha>
    for ColorAlpha<DstSpc, DstAlpha>
where
    SrcSpace: ColorSpace,
    DstSpc: ConvertFromRaw<SrcSpace>,
    SrcAlpha: AlphaState,
    DstAlpha: ConvertFromAlphaRaw<SrcAlpha> + AlphaState,
{
    type DstSpace = DstSpc;
    type DstAlpha = DstAlpha;
}

/// An object-safe trait implemented by both [`Color`] and [`DynamicColor`].
pub trait AnyColor {
    fn raw(&self) -> Vec3;
    fn space(&self) -> DynamicColorSpace;
    fn state(&self) -> DynamicState;

    /// Upcasts `self` into a [`DynamicColor`]
    fn dynamic(&self) -> DynamicColor {
        DynamicColor::new(self.raw(), self.space(), self.state())
    }
}

impl<'a> From<&'a dyn AnyColor> for DynamicColor {
    fn from(color: &'a dyn AnyColor) -> DynamicColor {
        color.dynamic()
    }
}

/// A type that implements this trait provides the ability to downcast from a dynamically-typed
/// color to a statically-typed [`Color`]. This is implemented for all types that implement [`AnyColor`]
pub trait DynColor {
    /// Attempt to convert to a typed `Color`. Returns an error if `self`'s color space and state do not match
    /// the given types.
    fn downcast<Spc: ColorSpace, St: State>(&self) -> ColorResult<Color<Spc, St>>;

    /// Convert to a typed `Color` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    fn downcast_unchecked<Spc: ColorSpace, St: State>(&self) -> Color<Spc, St>;
}

/// An object-safe trait implemented by both [`ColorAlpha`] and [`DynamicColorAlpha`]
pub trait AnyColorAlpha {
    fn raw(&self) -> Vec4;
    fn space(&self) -> DynamicColorSpace;
    fn alpha_state(&self) -> DynamicAlphaState;

    /// Upcasts `self` into a [`DynamicColorAlpha`]
    fn dynamic(&self) -> DynamicColorAlpha {
        DynamicColorAlpha::new(self.raw(), self.space(), self.alpha_state())
    }
}

/// A type that implements this trait provides the ability to downcast from a dynamically-typed
/// color to a statically-typed [`ColorAlpha`]. This is implemented for all types that implement [`AnyColorAlpha`]
pub trait DynColorAlpha {
    /// Attempt to downcast to a typed [`ColorAlpha`]. Returns an error if `self`'s color space and alpha state do not match
    /// the given types.
    fn downcast<Spc: ColorSpace, A: AlphaState>(&self) -> ColorResult<ColorAlpha<Spc, A>>;

    /// Downcast to a typed [`ColorAlpha`] without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    fn downcast_unchecked<Spc: ColorSpace, A: AlphaState>(&self) -> ColorAlpha<Spc, A>;
}
