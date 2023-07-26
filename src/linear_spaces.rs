use crate::{
    Color,
    component_structs::*,
    traits::*,
};

use glam::Vec3;

use kolor::details::color::*;

macro_rules! impl_conversion {
    ($space:ident to $dst_space:ident => None) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline(always)]
            fn linear_part_raw(color: &mut Vec3) {}
        }
    };
    ($space:ident to $dst_space:ident => $mat:ident) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline(always)]
            fn linear_part_raw(color: &mut Vec3) {
                *color = kolor::details::generated_matrices::$mat * *color;
            }
        }
    };
}

/* canonical conversion template
impl_conversion!(SPACENAME to SPACENAME         => None);
impl_conversion!(SPACENAME to Srgb              => PRIMARIES_WHITEPOINT_TO_BT_709_D65);
impl_conversion!(SPACENAME to CieXYZ            => PRIMARIES_WHITEPOINT_TO_CIE_XYZ_D65);
impl_conversion!(SPACENAME to Bt2020            => PRIMARIES_WHITEPOINT_TO_BT_2020_D65);
impl_conversion!(SPACENAME to AcesCg            => PRIMARIES_WHITEPOINT_TO_AP1_D60);
impl_conversion!(SPACENAME to Aces2065          => PRIMARIES_WHITEPOINT_TO_AP0_D60);
impl_conversion!(SPACENAME to DisplayP3         => PRIMARIES_WHITEPOINT_TO_P3_D65);
*/

/// A type representing the linear part of the sRGB color space.
pub struct Srgb;

impl LinearColorSpace for Srgb {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::BT_709;
    const WHITE_POINT: WhitePoint = WhitePoint::D65;
}

impl_conversion!(Srgb to Srgb              => None);
impl_conversion!(Srgb to CieXYZ            => BT_709_D65_TO_CIE_XYZ_D65);
impl_conversion!(Srgb to Bt2020            => BT_709_D65_TO_BT_2020_D65);
impl_conversion!(Srgb to AcesCg            => BT_709_D65_TO_AP1_D60);
impl_conversion!(Srgb to Aces2065          => BT_709_D65_TO_AP0_D60);
impl_conversion!(Srgb to DisplayP3         => BT_709_D65_TO_P3_D65);

/// A type representing the reference CIE XYZ 1931 color space.
pub struct CieXYZ;

impl LinearColorSpace for CieXYZ {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::CIE_XYZ;
    const WHITE_POINT: WhitePoint = WhitePoint::D65;
}

impl_conversion!(CieXYZ to CieXYZ            => None);
impl_conversion!(CieXYZ to Srgb              => CIE_XYZ_D65_TO_BT_709_D65);
impl_conversion!(CieXYZ to Bt2020            => CIE_XYZ_D65_TO_BT_2020_D65);
impl_conversion!(CieXYZ to AcesCg            => CIE_XYZ_D65_TO_AP1_D60);
impl_conversion!(CieXYZ to Aces2065          => CIE_XYZ_D65_TO_AP0_D60);
impl_conversion!(CieXYZ to DisplayP3         => CIE_XYZ_D65_TO_P3_D65);

/// A type representing the BT.2020 linear color space.
pub struct Bt2020;

impl LinearColorSpace for Bt2020 {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::BT_2020;
    const WHITE_POINT: WhitePoint = WhitePoint::D65;
}

/// A type representing the linear ACEScg color space.
pub struct AcesCg;

impl LinearColorSpace for AcesCg {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::AP1;
    const WHITE_POINT: WhitePoint = WhitePoint::D60;
}

/// A type representing the linear ACES 2065 (aka ACES archival) color space.
pub struct Aces2065;

impl LinearColorSpace for Aces2065 {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::AP0;
    const WHITE_POINT: WhitePoint = WhitePoint::D60;
}

/// A type representing the linear part of the Apple Display P3 color space.
pub struct DisplayP3;

impl LinearColorSpace for DisplayP3 {
    const PRIMARIES: RGBPrimaries = RGBPrimaries::P3;
    const WHITE_POINT: WhitePoint = WhitePoint::D65;
}
