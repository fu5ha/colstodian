use core::ops::{Add, Mul, Sub};

use crate::Color;

use glam::Vec3;
use kolor::details::color::{RGBPrimaries, WhitePoint};

/// A type that implements [`ColorEncoding`] represents a collection of metadata
/// that together defines how a color's data is stored and what the values of that data
/// actually mean.
///
/// You can see a list of all the built-in color encodings in [`crate::details::encodings`].
pub trait ColorEncoding: Sized + 'static {
    /// The raw data representation used by this encoding.
    ///
    /// You can see a list of all the reprs used by the built-in encodings
    /// in [`crate::details::reprs`].
    type Repr: ColorRepr;

    /// The 'bag of components' this color encoding uses.
    ///
    /// You can see a list of all the built-in component structs in
    /// [`crate::details::component_structs`].
    type ComponentStruct: ComponentStructFor<Self::Repr>;

    /// The [`LinearColorSpace`] used by this encoding.
    ///
    /// You can see all the built-in linear spaces in
    /// [`crate::details::linear_spaces`]
    type LinearSpace: LinearColorSpace;

    /// Used in `Debug` and `Display` implementations.
    const NAME: &'static str;

    /// Convert from `Self::Repr` to a `glam::Vec3` in the `Self::LinearSpace` color space and a separate
    /// (not pre-multiplied) alpha component. If this encoding does not have alpha, return 1.0.
    fn src_transform_raw(repr: Self::Repr) -> (Vec3, f32);

    /// Convert from a `glam::Vec3` in `Self::LinearSpace` and separate alpha component to a `Self::Repr` fully
    /// encoded in `Self`'s color encoding. If this encoding does not have alpha, you can disregard it.
    fn dst_transform_raw(raw: Vec3, alpha: f32) -> Self::Repr;
}

/// Implementing this trait for a struct marks that it is safe to pointer cast `Repr` as `Self`.
///
/// # Safety
///
/// In order to implement this trait, it must be safe to perform the casts implied by the `cast` and
/// `cast_mut` functions.
pub unsafe trait ComponentStructFor<Repr: ColorRepr>:
    Sized + Clone + Copy + 'static
{
    fn cast(repr: &Repr) -> &Self;
    fn cast_mut(repr: &mut Repr) -> &mut Self;
}

/// Implemented by the raw data representation of a color encoding.
///
/// You can see a list of all the reprs used by the built-in encodings
/// in [`crate::details::reprs`].
pub trait ColorRepr: Sized + Clone + Copy + 'static {
    /// The type of a single element of this repr
    type Element: Sized + Clone + Copy + 'static;
}

/// Implemented by color encodings that can do alpha compositing.
pub trait AlphaOver: ColorEncoding {
    fn composite(over: Color<Self>, under: Color<Self>) -> Color<Self>;
}

/// Implemented by color encodings that can perform saturate-style clamping.
pub trait Saturate: ColorEncoding {
    fn saturate(repr: Self::Repr) -> Self::Repr;
}

/// Implemented by color encodings which can perform linear interpolation between colors.
/// The interpolation is not necessarily perceptually-linear, it is just linear within the
/// given encoding.
pub trait LinearInterpolate
where
    Self: ColorEncoding + WorkingEncoding,
{
    fn lerp(from: Color<Self>, to: Color<Self>, factor: f32) -> Color<Self>;
}

impl<E> LinearInterpolate for E
where
    E: ColorEncoding + WorkingEncoding,
    E::Repr: Add<Output = E::Repr> + Sub<Output = E::Repr> + Mul<f32, Output = E::Repr>,
{
    #[inline]
    fn lerp(from: Color<Self>, to: Color<Self>, factor: f32) -> Color<Self> {
        Color {
            repr: from.repr + ((to.repr - from.repr) * factor),
        }
    }
}

/// Implemented by color encodings which are designed to be perceptually-uniform. In general these encodings
/// will produce more visually pleasing results when blending between colors (for example, creating gradients)
/// in many situations. However they are certainly not, silver bullets, and often don't fully deliver on the
/// promise of perceptual uniformity.
pub trait PerceptualEncoding: ColorEncoding + WorkingEncoding {}

/// Marks a type as representing a color encoding in which it makes sense to be able to perform mathematical
/// operations on the contained color values directly.
pub trait WorkingEncoding: ColorEncoding {}

/// Implemented by "color" encodings which actually represent a 'quasi radiance' value, i.e. the intensity of light
/// along a given ray (or radiometric flux of the light per unit solid angle, to be pedantic).
/// 
/// If you're confused what I mean by this, I suggest reading this blog post: https://fusha.moe/blog/posts/whirlwind-games-color/
/// 
/// Calling values like this a "color" is a bit of a stretch of the definition, but it's a common enough thing in
/// computer graphics to want to work with values like this and turn them into *actual* colors, so it deserves to
/// be incorporated in this library somehow.
/// 
/// Color encodings that implement this trait cannot be directly converted to or from most other encodings. Instead you can use
/// the [`Color::render_image`] which allows you to choose an "image formation"/"display rendering transform" strategy to use
/// during the conversion to a compatible "true color" space.
pub trait QuasiRadianceEncoding: ColorEncoding + WorkingEncoding {
    type BaseLinearSpace: LinearColorSpace;
}

/// Implemented by "color" encodings which actually represent a 'quasi reflectance' value, i.e. each component represents the
/// *percentage* of incoming [quasi-radiant flux][QuasiRadianceEncoding] in the same channel that gets reflected (not absorbed)
/// by a given material in a rendering context.
pub trait QuasiReflectanceEncoding: ColorEncoding {
    type BaseLinearSpace: LinearColorSpace;
}

/// A type that implements [`LinearColorSpace`] represents a color space which can be defined by a *linear transformation only*
/// (i.e. a 3x3 matrix multiplication) from the CIE XYZ color space.
///
/// A linear color space is defined by the combination of a set of [Primaries][RGBPrimaries] and a [White Point][WhitePoint].
///
/// You can see all the built-in linear spaces in [`crate::details::linear_spaces`]
pub trait LinearColorSpace {
    const PRIMARIES: RGBPrimaries;
    const WHITE_POINT: WhitePoint;
}

/// A trait that marks `Self` as being a color encoding which is able to be directly converted from `SrcEnc`,
/// as well as allowing some hooks to perform extra mapping during the conversion if necessary. This is the trait that
/// unlocks the [`.convert::<E>`][Color::convert] method on [`Color`].
///
/// In order to be able to [`convert`][Color::convert] from [`Color<EncodingA>`] to [`Color<EncodingB>`], `EncodingB`
/// must implement [`ConvertFrom<EncodingA>`].
///
/// If this trait is not implemented for a pair of encodings, then a direct conversion without input or choice from the user
/// is not possible, and a conversion between the encodings will need to be performed manually or in more than one step.
pub trait ConvertFrom<SrcEnc>
where
    SrcEnc: ColorEncoding,
    Self: ColorEncoding,
    Self::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
{
    /// If required or desired, perform a mapping of some kind to the input
    /// before it undergoes its source transform. This may be desirable to perform some form of
    /// gamut mapping if the src encoding has a larger size of representable colors than the dst encoding.
    #[inline(always)]
    fn map_src(_src: &mut SrcEnc::Repr) {}
}

impl<E> ConvertFrom<E> for E
where
    E: ColorEncoding,
    E::LinearSpace: LinearConvertFromRaw<E::LinearSpace>,
{
}

/// The complement of [`ConvertFrom`], which is blanket impl'd in the same way that [`Into`] is implemented
/// as the converse of [`From`] by the standard library.
pub trait ConvertTo<DstEnc>
where
    DstEnc: ColorEncoding,
    Self: ColorEncoding,
    DstEnc::LinearSpace: LinearConvertFromRaw<Self::LinearSpace>,
{
    /// If required or desired, perform a mapping of some kind to the input
    /// before it undergoes its source transform. This may be desirable to perform some form of
    /// gamut mapping if the src encoding has a larger size of representable colors than the dst encoding.
    #[inline(always)]
    fn map_src(_src: &mut Self::Repr) {}
}

impl<SrcEnc, DstEnc> ConvertTo<DstEnc> for SrcEnc
where
    DstEnc: ColorEncoding,
    SrcEnc:ColorEncoding,
    DstEnc::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
    DstEnc: ConvertFrom<SrcEnc>,
{
    #[inline(always)]
    fn map_src(src: &mut SrcEnc::Repr) {
        <DstEnc as ConvertFrom<SrcEnc>>::map_src(src)
    }
}

/// Performs the raw conversion from the [`LinearColorSpace`] represented by `SrcSpc` to
/// the [`LinearColorSpace`] represented by `Self`.
pub trait LinearConvertFromRaw<SrcSpace: LinearColorSpace>: LinearColorSpace {
    fn linear_part_raw(raw: &mut Vec3);
}
