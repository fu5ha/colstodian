use crate::component_structs::*;
use crate::linear_spaces;
use crate::reprs::*;
use crate::traits::*;
use crate::Color;

use glam::Vec3;
use glam::Vec4;
use glam::Vec4Swizzles;
use kolor::details::color::WhitePoint;
use kolor::details::transform;

#[inline(always)]
fn u8_to_f32(x: u8) -> f32 {
    x as f32 / 255.0
}

#[inline(always)]
fn f32_to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0) as u8
}

#[doc = include_str!("descriptions/srgb_u8.md")]
pub struct SrgbU8;

impl Color<SrgbU8> {
    /// Create a [`Color`] in the [`SrgbU8`] encoding.
    ///
    /// **If you don't know what you're doing and you have RGB values from a color picker that vary
    /// from `0-255`, use this.**
    ///
    /// If you're not sure, see [the `SrgbU8` encoding docs][SrgbU8] for more info.
    #[inline(always)]
    pub const fn srgb_u8(r: u8, g: u8, b: u8) -> Self {
        Color::from_repr([r, g, b])
    }
}

impl ColorEncoding for SrgbU8 {
    type Repr = U8Repr;

    type ComponentStruct = Rgb<u8>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbU8";

    #[inline]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let [x, y, z] = repr;
        let raw_electro = Vec3::new(u8_to_f32(x), u8_to_f32(y), u8_to_f32(z));
        let optical = transform::sRGB_eotf(raw_electro, WhitePoint::D65);
        (optical, 1.0)
    }

    #[inline]
    fn dst_transform_raw(raw: glam::Vec3, _: f32) -> Self::Repr {
        let electro = transform::sRGB_oetf(raw, WhitePoint::D65);
        let repr = [
            f32_to_u8(electro.x),
            f32_to_u8(electro.y),
            f32_to_u8(electro.z),
        ];
        repr
    }
}

impl ConvertFrom<SrgbF32> for SrgbU8 {}
impl ConvertFrom<SrgbAU8> for SrgbU8 {}
impl ConvertFrom<SrgbAF32> for SrgbU8 {}
impl ConvertFrom<SrgbAU8Premultiplied> for SrgbU8 {}
impl ConvertFrom<LinearSrgb> for SrgbU8 {}
impl ConvertFrom<LinearSrgbA> for SrgbU8 {}
impl ConvertFrom<LinearSrgbAPremultiplied> for SrgbU8 {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for SrgbU8 {}

#[doc = include_str!("descriptions/srgb_f32.md")]
pub struct SrgbF32;

impl Color<SrgbF32> {
    /// Create a [`Color`] in the [`SrgbF32`] encoding.
    ///
    /// **If you don't know what you're doing and you have RGB values from a color picker that vary
    /// from `0.0..=1.0`, use this.**
    ///
    /// If you're not sure, see [the `SrgbF32` encoding docs][SrgbF32] for more info.
    #[inline(always)]
    pub const fn srgb_f32(r: f32, g: f32, b: f32) -> Self {
        Color::from_repr(Vec3::new(r, g, b))
    }
}

impl ColorEncoding for SrgbF32 {
    type Repr = F32Repr;

    type ComponentStruct = Rgb<f32>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbF32";

    #[inline]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let optical = transform::sRGB_eotf(repr, WhitePoint::D65);
        (optical, 1.0)
    }

    #[inline]
    fn dst_transform_raw(raw: glam::Vec3, _: f32) -> Self::Repr {
        let electro = transform::sRGB_oetf(raw, WhitePoint::D65);
        electro
    }
}

impl ConvertFrom<SrgbU8> for SrgbF32 {}
impl ConvertFrom<SrgbAU8> for SrgbF32 {}
impl ConvertFrom<SrgbAF32> for SrgbF32 {}
impl ConvertFrom<SrgbAU8Premultiplied> for SrgbF32 {}
impl ConvertFrom<LinearSrgb> for SrgbF32 {}
impl ConvertFrom<LinearSrgbA> for SrgbF32 {}
impl ConvertFrom<LinearSrgbAPremultiplied> for SrgbF32 {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for SrgbF32 {}

#[doc = include_str!("descriptions/srgba_u8.md")]
pub struct SrgbAU8;

impl Color<SrgbAU8> {
    /// Create a [`Color`] in the [`SrgbAU8`] encoding.
    ///
    /// **If you don't know what you're doing and you have RGBA values from a color picker that vary
    /// from `0-255`, use this.**
    ///
    /// If you're not sure, see [the `SrgbAU8` encoding docs][SrgbAU8] for more info.
    #[inline(always)]
    pub const fn srgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_repr([r, g, b, a])
    }
}

impl ColorEncoding for SrgbAU8 {
    type Repr = U8ARepr;

    type ComponentStruct = RgbA<u8>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbAU8";

    #[inline]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let [x, y, z, a] = repr;
        let raw_electro = Vec3::new(u8_to_f32(x), u8_to_f32(y), u8_to_f32(z));
        let optical = transform::sRGB_eotf(raw_electro, WhitePoint::D65);
        let a = u8_to_f32(a);
        (optical, a)
    }

    #[inline]
    fn dst_transform_raw(raw: glam::Vec3, alpha: f32) -> Self::Repr {
        let electro = transform::sRGB_oetf(raw, WhitePoint::D65);
        let repr = [
            f32_to_u8(electro.x),
            f32_to_u8(electro.y),
            f32_to_u8(electro.z),
            f32_to_u8(alpha),
        ];
        repr
    }
}

impl ConvertFrom<SrgbU8> for SrgbAU8 {}
impl ConvertFrom<SrgbF32> for SrgbAU8 {}
impl ConvertFrom<SrgbAF32> for SrgbAU8 {}
impl ConvertFrom<SrgbAU8Premultiplied> for SrgbAU8 {}
impl ConvertFrom<LinearSrgb> for SrgbAU8 {}
impl ConvertFrom<LinearSrgbA> for SrgbAU8 {}
impl ConvertFrom<LinearSrgbAPremultiplied> for SrgbAU8 {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for SrgbAU8 {}

#[doc = include_str!("descriptions/srgba_f32.md")]
pub struct SrgbAF32;

impl Color<SrgbAF32> {
    /// Create a [`Color`] in the [`SrgbAF32`] encoding.
    ///
    /// **If you don't know what you're doing and you have RGB values from a color picker that vary
    /// from `0.0..=1.0`, use this.**
    ///
    /// If you're not sure, see [the `SrgbAF32` encoding docs][SrgbAF32] for more info.
    #[inline(always)]
    pub const fn srgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color::from_repr(Vec4::new(r, g, b, a))
    }
}

impl ColorEncoding for SrgbAF32 {
    type Repr = F32ARepr;

    type ComponentStruct = RgbA<f32>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbAF32";

    #[inline]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let optical = transform::sRGB_eotf(repr.xyz(), WhitePoint::D65);
        (optical, repr.w)
    }

    #[inline]
    fn dst_transform_raw(raw: glam::Vec3, alpha: f32) -> Self::Repr {
        let electro = transform::sRGB_oetf(raw, WhitePoint::D65);
        electro.extend(alpha)
    }
}

impl ConvertFrom<SrgbU8> for SrgbAF32 {}
impl ConvertFrom<SrgbAU8> for SrgbAF32 {}
impl ConvertFrom<SrgbF32> for SrgbAF32 {}
impl ConvertFrom<SrgbAU8Premultiplied> for SrgbAF32 {}
impl ConvertFrom<LinearSrgb> for SrgbAF32 {}
impl ConvertFrom<LinearSrgbA> for SrgbAF32 {}
impl ConvertFrom<LinearSrgbAPremultiplied> for SrgbAF32 {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for SrgbAF32 {}

/// The fully-encoded form of the sRGB color encoding standard, with *premultiplied* alpha component.
///
/// Premultiplied means that the color components are already multiplied by the alpha component. Such multiplication
/// happens *before* the sRGB OETF is applied.
///
/// This is not a common way for humans to specify colors directly, but is a moderately common way to encode
/// textures before uploading them to the GPU or otherwise using them in a rendering pipeline.
///
/// This color encoding is defined as the strict sRGB color encoding standard, with
/// OETF applied and encoded into 8 bits per component. The alpha component is linearly encoded
/// into 8 bits, i.e. the sRGB OETF is not applied.
pub struct SrgbAU8Premultiplied;

impl ColorEncoding for SrgbAU8Premultiplied {
    type Repr = U8ARepr;

    type ComponentStruct = RgbA<u8>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbAU8Premultiplied";

    #[inline]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let [x, y, z, a] = repr;
        let raw_electro = Vec3::new(u8_to_f32(x), u8_to_f32(y), u8_to_f32(z));
        let optical = transform::sRGB_eotf(raw_electro, WhitePoint::D65);
        let a = u8_to_f32(a);
        let separated = optical / a;
        (separated, a)
    }

    #[inline]
    fn dst_transform_raw(raw: glam::Vec3, alpha: f32) -> Self::Repr {
        let premultiplied = raw * alpha;
        let electro = transform::sRGB_oetf(premultiplied, WhitePoint::D65);
        let repr = [
            f32_to_u8(electro.x),
            f32_to_u8(electro.y),
            f32_to_u8(electro.z),
            f32_to_u8(alpha),
        ];
        repr
    }
}

impl ConvertFrom<SrgbU8> for SrgbAU8Premultiplied {}
impl ConvertFrom<SrgbF32> for SrgbAU8Premultiplied {}
impl ConvertFrom<SrgbAF32> for SrgbAU8Premultiplied {}
impl ConvertFrom<SrgbAU8> for SrgbAU8Premultiplied {}
impl ConvertFrom<LinearSrgb> for SrgbAU8Premultiplied {}
impl ConvertFrom<LinearSrgbA> for SrgbAU8Premultiplied {}
impl ConvertFrom<LinearSrgbAPremultiplied> for SrgbAU8Premultiplied {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for SrgbAU8Premultiplied {}

impl AlphaOver for SrgbAU8Premultiplied {
    fn composite(over: Color<Self>, under: Color<Self>) -> Color<Self> {
        let over = over.convert::<LinearSrgbAPremultiplied>();
        let under = under.convert::<LinearSrgbAPremultiplied>();
        let comp = over.alpha_over(under);
        comp.convert::<Self>()
    }
}

/// The linear form of the sRGB color encoding standard.
///
/// This is a moderately rare way to specify color values.
///
/// If you have three f32s which are *not* directly related to the u8 form, or you otherwise know should be
/// "linear rgb" values, then this is the encoding you have. If you instead have four values with an alpha
/// component where the alpha component varies independently of the color components, you have [`LinearSrgbA`] values.
/// If you have four values with an alpha component and the rgb components are modified directly when the alpha component
/// changes as well, you have [`LinearSrgbAPremultiplied`] values.
pub struct LinearSrgb;

impl Color<LinearSrgb> {
    /// Create a [`Color`] in the [`LinearSrgb`] encoding.
    ///
    /// If you're not sure, you should probably use [`Color::srgb_f32`] instead.
    /// See [the `LinearSrgb` encoding docs][LinearSrgb] for more info.
    #[inline(always)]
    pub fn linear_srgb(r: f32, g: f32, b: f32) -> Self {
        Color::from_repr(Vec3::new(r, g, b))
    }
}

impl ColorEncoding for LinearSrgb {
    type Repr = F32Repr;

    type ComponentStruct = Rgb<f32>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "LinearSrgb";

    #[inline(always)]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        (repr, 1.0)
    }

    #[inline(always)]
    fn dst_transform_raw(raw: glam::Vec3, _: f32) -> Self::Repr {
        raw
    }
}

impl ConvertFrom<SrgbU8> for LinearSrgb {}
impl ConvertFrom<SrgbF32> for LinearSrgb {}
impl ConvertFrom<SrgbAU8> for LinearSrgb {}
impl ConvertFrom<SrgbAF32> for LinearSrgb {}
impl ConvertFrom<SrgbAU8Premultiplied> for LinearSrgb {}
impl ConvertFrom<LinearSrgbA> for LinearSrgb {}
impl ConvertFrom<LinearSrgbAPremultiplied> for LinearSrgb {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for LinearSrgb {}

impl WorkingEncoding for LinearSrgb {}

/// The linear form of the sRGB color encoding standard with a separate alpha component.
///
/// This is a moderately common way to specify color values.
///
/// If you have four f32s which are *not* directly related to the u8 form, or you otherwise know should be
/// "linear rgb" values, and the alpha component varies independently of the color componewnts,
/// then this is the encoding you have. If you instead have three values, you have [`LinearSrgb`] values.
/// If you have four values with an alpha component and the rgb components are modified directly when the alpha component
/// changes as well, you have [`LinearSrgbAPremultiplied`] values.
pub struct LinearSrgbA;

impl Color<LinearSrgbA> {
    /// Create a [`Color`] in the [`LinearSrgbA`] encoding.
    ///
    /// If you're not sure, you should probably use [`Color::srgba_f32`] instead.
    /// See [the `LinearSrgbA` encoding docs][LinearSrgbA] for more info.
    #[inline(always)]
    pub fn linear_srgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color::from_repr(Vec4::new(r, g, b, a))
    }
}

impl ColorEncoding for LinearSrgbA {
    type Repr = F32ARepr;

    type ComponentStruct = RgbA<f32>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "LinearSrgbA";

    #[inline(always)]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        (repr.xyz(), repr.w)
    }

    #[inline(always)]
    fn dst_transform_raw(raw: glam::Vec3, alpha: f32) -> Self::Repr {
        raw.extend(alpha)
    }
}

impl ConvertFrom<SrgbU8> for LinearSrgbA {}
impl ConvertFrom<SrgbF32> for LinearSrgbA {}
impl ConvertFrom<SrgbAU8> for LinearSrgbA {}
impl ConvertFrom<SrgbAF32> for LinearSrgbA {}
impl ConvertFrom<SrgbAU8Premultiplied> for LinearSrgbA {}
impl ConvertFrom<LinearSrgb> for LinearSrgbA {}
impl ConvertFrom<LinearSrgbAPremultiplied> for LinearSrgbA {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for LinearSrgbA {}

impl WorkingEncoding for LinearSrgbA {}

impl AlphaOver for LinearSrgbA {
    fn composite(over: Color<Self>, under: Color<Self>) -> Color<Self> {
        let over = over.convert::<LinearSrgbAPremultiplied>();
        let under = under.convert::<LinearSrgbAPremultiplied>();
        let comp = over.alpha_over(under);
        comp.convert::<Self>()
    }
}

/// The linear form of the sRGB color encoding standard with a *premultiplied* alpha component.
///
/// "Premultiplied" alpha means that the value of the color components has been multiplied by the
/// alpha component. This operation is unintuitive when specifying color values, but it is the
/// "most correct" way to store color values with an alpha component when performing operations
/// like blending and compositing on them.
///
/// This is a relatively rare way to specify color values.
///
/// If you have four f32s which are *not* directly related to the u8 form, or you otherwise know should be
/// "linear rgb" values, and the alpha component varies independently of the color componewnts,
/// then this is the encoding you have. If you instead have three values, you have [`LinearSrgb`] values.
/// If you have four values with an alpha component and the rgb components are modified directly when the alpha component
/// changes as well, you have [`LinearSrgbAPremultiplied`] values.
pub struct LinearSrgbAPremultiplied;

impl Color<LinearSrgbAPremultiplied> {
    /// Create a [`Color`] in the [`LinearSrgbAPremultiplied`] encoding.
    ///
    /// "Premultiplied" alpha means that the value of the color components has been multiplied by the
    /// alpha component. This operation is unintuitive when specifying color values, but it is the
    /// "most correct" way to store color values with an alpha component when performing operations
    /// like blending and compositing on them.
    ///
    /// If you're not sure, see [the `LinearSrgbA` encoding docs][LinearSrgbA] for more info.
    #[inline(always)]
    pub fn linear_srgba_premultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color::from_repr(Vec4::new(r, g, b, a))
    }
}

impl ColorEncoding for LinearSrgbAPremultiplied {
    type Repr = F32ARepr;

    type ComponentStruct = RgbA<f32>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "LinearSrgbAPremultiplied";

    #[inline(always)]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let separated = repr.xyz() / repr.w;
        (separated, repr.w)
    }

    #[inline(always)]
    fn dst_transform_raw(raw: glam::Vec3, alpha: f32) -> Self::Repr {
        let premultiplied = raw * alpha;
        premultiplied.extend(alpha)
    }
}

impl ConvertFrom<SrgbU8> for LinearSrgbAPremultiplied {}
impl ConvertFrom<SrgbF32> for LinearSrgbAPremultiplied {}
impl ConvertFrom<SrgbAU8> for LinearSrgbAPremultiplied {}
impl ConvertFrom<SrgbAF32> for LinearSrgbAPremultiplied {}
impl ConvertFrom<SrgbAU8Premultiplied> for LinearSrgbAPremultiplied {}
impl ConvertFrom<LinearSrgbA> for LinearSrgbAPremultiplied {}
impl ConvertFrom<LinearSrgb> for LinearSrgbAPremultiplied {}
// TODO: oklab gamut clipping
impl ConvertFrom<Oklab> for LinearSrgbAPremultiplied {}

impl AlphaOver for LinearSrgbAPremultiplied {
    #[inline]
    fn composite(over: Color<Self>, under: Color<Self>) -> Color<Self> {
        Color::from_repr(over.repr + under.repr * (1.0 - over.repr.w))
    }
}

/// A 32-bit-per-component version of the Oklab perceptually-uniform color space.
pub struct Oklab;

impl Color<Oklab> {
    /// Create a [`Color`] in the [`Oklab`] color encoding.
    ///
    /// This is fairly rare, it would be more common to specify colors in another color encoding like
    /// [`SrgbU8`] and then convert them to [`Oklab`] to blend them together.
    #[inline(always)]
    pub fn oklab(l: f32, a: f32, b: f32) -> Self {
        Color::from_repr(Vec3::new(l, a, b))
    }
}

impl ColorEncoding for Oklab {
    type Repr = F32Repr;

    type ComponentStruct = Lab<f32>;

    type LinearSpace = linear_spaces::CieXYZ;

    const NAME: &'static str = "Oklab";

    #[inline(always)]
    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let xyz = transform::Oklab_to_XYZ(repr, WhitePoint::D65);
        (xyz, 1.0)
    }

    #[inline(always)]
    fn dst_transform_raw(raw: glam::Vec3, _: f32) -> Self::Repr {
        let oklab = transform::XYZ_to_Oklab(raw, WhitePoint::D65);
        oklab
    }
}

impl ConvertFrom<SrgbU8> for Oklab {}
impl ConvertFrom<SrgbF32> for Oklab {}
impl ConvertFrom<SrgbAU8> for Oklab {}
impl ConvertFrom<SrgbAF32> for Oklab {}
impl ConvertFrom<SrgbAU8Premultiplied> for Oklab {}
impl ConvertFrom<LinearSrgb> for Oklab {}
impl ConvertFrom<LinearSrgbA> for Oklab {}
impl ConvertFrom<LinearSrgbAPremultiplied> for Oklab {}

impl WorkingEncoding for Oklab {}
impl PerceptualEncoding for Oklab {}
