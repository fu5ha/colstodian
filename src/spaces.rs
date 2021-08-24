use crate::{
    ColorSpace, Color, ColorAlpha, Display, Separate, Premultiplied,
    component_structs::{Rgb, ICtCp, ColAlpha, LCh, Lab, Xyz},
    traits::*
};

#[cfg(not(target_arch = "spirv"))]
use crate::{ColorU8, ColorU8Alpha};

use glam::{Vec3, Vec4};

use core::ops::*;
use core::fmt;

pub use kolor::ColorSpace as DynamicColorSpace;

/// Color spaces defined as data.
pub mod dynamic_spaces {
    use super::*;
    use kolor::details::color::*;

    pub use kolor::spaces::*;

    pub const ENCODED_ACES_CG_SRGB: DynamicColorSpace =
        DynamicColorSpace::new(RGBPrimaries::AP1, WhitePoint::D60, TransformFn::sRGB);
}

macro_rules! impl_cint {
    ($t:ident, $cint_ty:ident, $color_ty:ident, $color_alpha_ty:ident, $space:ident) => {
        impl From<cint::$cint_ty<$t>> for $color_ty<$space, Display> {
            fn from(color: cint::$cint_ty<$t>) -> $color_ty<$space, Display> {
                let arr: [$t; 3] = color.into();
                $color_ty::from(arr)
            }
        }

        impl From<$color_ty<$space, Display>> for cint::$cint_ty<$t> {
            fn from(color: $color_ty<$space, Display>) -> cint::$cint_ty<$t> {
                From::from(*color.as_ref())
            }
        }

        impl<St> From<cint::Alpha<cint::$cint_ty<$t>>> for $color_alpha_ty<$space, St, Separate> {
            fn from(color: cint::Alpha<cint::$cint_ty<$t>>) -> $color_alpha_ty<$space, St, Separate> {
                let arr: [$t; 4] = color.into();
                $color_alpha_ty::from(arr)
            }
        }

        impl<St> From<$color_alpha_ty<$space, St, Separate>> for cint::Alpha<cint::$cint_ty<$t>> {
            fn from(color: $color_alpha_ty<$space, St, Separate>) -> cint::Alpha<cint::$cint_ty<$t>> {
                From::from(*color.as_ref())
            }
        }

        impl<St> From<cint::PremultipliedAlpha<cint::$cint_ty<$t>>> for $color_alpha_ty<$space, St, Premultiplied> {
            fn from(color: cint::PremultipliedAlpha<cint::$cint_ty<$t>>) -> $color_alpha_ty<$space, St, Premultiplied> {
                let arr: [$t; 4] = color.into();
                $color_alpha_ty::from(arr)
            }
        }

        impl<St> From<$color_alpha_ty<$space, St, Premultiplied>> for cint::PremultipliedAlpha<cint::$cint_ty<$t>> {
            fn from(color: $color_alpha_ty<$space, St, Premultiplied>) -> cint::PremultipliedAlpha<cint::$cint_ty<$t>> {
                From::from(*color.as_ref())
            }
        }

        impl cint::ColorInterop for $color_ty<$space, Display> {
            type CintTy = cint::$cint_ty<$t>;
        }

        impl<St> cint::ColorInterop for $color_alpha_ty<$space, St, Separate> {
            type CintTy = cint::Alpha<cint::$cint_ty<$t>>;
        }

        impl<St> cint::ColorInterop for $color_alpha_ty<$space, St, Premultiplied> {
            type CintTy = cint::PremultipliedAlpha<cint::$cint_ty<$t>>;
        }
    }
}
macro_rules! impl_color_space_inner {
    {
        $space:ident is $dynamic_space:ident,
        $(cint is $cint_ty:ident,)?
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl ColorSpace for $space {
            /// The [`DynamicColorSpace`] that this type represents.
            const SPACE: DynamicColorSpace = dynamic_spaces::$dynamic_space;

            /// The closest linear color space to this space.
            type LinearSpace = $lin_space;

            /// The 'bag of components' that this color space uses.
            type ComponentStruct = $derefs_to;
        }

        impl<SrcSpace, St> ColorConversionQuery<SrcSpace, St> for $space
        where
            SrcSpace: ColorSpace,
            Self: ConvertFromRaw<SrcSpace>,
            St: State,
        {
            type DstSpace = Self;
        }

        impl<SrcSpace, SrcAlpha> ColorAlphaConversionQuery<SrcSpace, SrcAlpha> for $space
        where
            SrcSpace: ColorSpace,
            Self: ConvertFromRaw<SrcSpace>,
            SrcAlpha: AlphaState,
        {
            type DstSpace = Self;
            type DstAlpha = SrcAlpha;
        }

        impl Default for $space {
            fn default() -> Self {
                Self {}
            }
        }

        #[cfg(not(target_arch = "spirv"))]
        impl fmt::Display for $space {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $display)
            }
        }

        impl<St> Deref for Color<$space, St> {
            type Target = <$space as ColorSpace>::ComponentStruct;

            /// Test
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(&self.raw as *const Vec3 as *const Self::Target) }
            }
        }

        impl<St> DerefMut for Color<$space, St> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(&mut self.raw as *mut Vec3 as *mut Self::Target) }
            }
        }

        impl<St, A> Deref for ColorAlpha<$space, St, A> {
            type Target = ColAlpha<<$space as ColorSpace>::ComponentStruct>;

            /// Test
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(&self.raw as *const Vec4 as *const Self::Target) }
            }
        }

        impl<St, A> DerefMut for ColorAlpha<$space, St, A> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(&mut self.raw as *mut Vec4 as *mut Self::Target) }
            }
        }

        $(
            impl_cint!(f32, $cint_ty, Color, ColorAlpha, $space);
            #[cfg(not(target_arch = "spirv"))]
            impl_cint!(u8, $cint_ty, ColorU8, ColorU8Alpha, $space);
        )?
    };
}

macro_rules! impl_color_space {
    {
        $space:ident is $dynamic_space:ident and Nonlinear,
        $(cint is $cint_ty:ident,)?
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            $(cint is $cint_ty,)?
            LinearSpace is $lin_space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl NonlinearColorSpace for $space {}

        impl WorkingColorSpace for $space {}
    };
    {
        $space:ident is $dynamic_space:ident and Nonlinear,
        $(cint is $cint_ty:ident,)?
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
        Encodes from $decoded:ident,
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            $(cint is $cint_ty,)?
            LinearSpace is $lin_space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl NonlinearColorSpace for $space {}

        impl EncodedColorSpace for $space {
            type DecodedSpace = $decoded;
        }

    };
    {
        $space:ident is $dynamic_space:ident and Linear,
        $(cint is $cint_ty:ident,)?
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            $(cint is $cint_ty,)?
            LinearSpace is $space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl LinearColorSpace for $space {}

        impl WorkingColorSpace for $space {}
    };
}

macro_rules! impl_conversion {
    ($space:ident to $dst_space:ident => None, None, None) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                color
            }
        }
    };
    ($space:ident to $dst_space:ident => $src_transform_fn:ident, None, None) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$src_transform_fn(color, $space::SPACE.white_point())
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                color
            }
        }
    };
    ($space:ident to $dst_space:ident => None, $mat:ident, None) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                kolor::details::generated_matrices::$mat * color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                color
            }
        }
    };
    ($space:ident to $dst_space:ident => None, None, $dst_transform_fn:ident) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$dst_transform_fn(color, $dst_space::SPACE.white_point())
            }
        }
    };
    ($space:ident to $dst_space:ident => $src_transform_fn:ident, $mat:ident, None) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$src_transform_fn(color, $space::SPACE.white_point())
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                kolor::details::generated_matrices::$mat * color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                color
            }
        }
    };
    ($space:ident to $dst_space:ident => None, $mat:ident, $dst_transform_fn:ident) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                kolor::details::generated_matrices::$mat * color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$dst_transform_fn(color, $dst_space::SPACE.white_point())
            }
        }
    };
    ($space:ident to $dst_space:ident => $src_transform_fn:ident, None, $dst_transform_fn:ident) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$src_transform_fn(color, $space::SPACE.white_point())
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$dst_transform_fn(color, $dst_space::SPACE.white_point())
            }
        }
    };
    ($space:ident to $dst_space:ident => $src_transform_fn:ident, $mat:ident, $dst_transform_fn:ident) => {
        impl ConvertFromRaw<$space> for $dst_space {
            #[inline]
            fn src_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$src_transform_fn(color, $space::SPACE.white_point())
            }
            #[inline]
            fn linear_part_raw(color: Vec3) -> Vec3 {
                kolor::details::generated_matrices::$mat * color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$dst_transform_fn(color, $dst_space::SPACE.white_point())
            }
        }
    };
}

#[cfg(not(target_arch = "spirv"))]
macro_rules! impl_as_u8_array {
    ($space:ident: $cint_ty:ident) => {
        impl AsU8 for $space {}
    }
}

/* Canonical conversion template
impl_conversion!(SPACENAME to SPACENAME        => None, None, None);
impl_conversion!(SPACENAME to LinearSrgb        => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_709_D65, None);
impl_conversion!(SPACENAME to AcesCg            => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_AP1_D60, None);
impl_conversion!(SPACENAME to Aces2065          => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_AP0_D60, None);
impl_conversion!(SPACENAME to DisplayP3         => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_P3_D65, None);
impl_conversion!(SPACENAME to CieXyz            => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_CIE_XYZ_D65, None);
impl_conversion!(SPACENAME to Bt2020            => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_2020_D65, None);
impl_conversion!(SPACENAME to Oklab             => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(SPACENAME to Oklch             => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(SPACENAME to ICtCpPQ           => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(SPACENAME to EncodedAcesCgSrgb => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_AP1_D60, sRGB_oetf);
impl_conversion!(SPACENAME to EncodedBt2020     => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(SPACENAME to EncodedBt2100PQ   => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(SPACENAME to EncodedDisplayP3  => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_P3_D65, sRGB_oetf);
impl_conversion!(SPACENAME to EncodedSrgb       => SPACE_EOTF, PRIMARIES_WHITEPOINT_TO_BT_709_D65, sRGB_oetf);
*/

/// A type representing the [linear sRGB][dynamic_spaces::LINEAR_SRGB] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct LinearSrgb;

impl_color_space! {
    LinearSrgb is LINEAR_SRGB and Linear,
    cint is LinearSrgb,
    Derefs as Rgb,
    Displays as "Linear sRGB",
}

impl_conversion!(LinearSrgb to LinearSrgb        => None, None, None);
impl_conversion!(LinearSrgb to AcesCg            => None, BT_709_D65_TO_AP1_D60, None);
impl_conversion!(LinearSrgb to Aces2065          => None, BT_709_D65_TO_AP0_D60, None);
impl_conversion!(LinearSrgb to DisplayP3         => None, BT_709_D65_TO_P3_D65, None);
impl_conversion!(LinearSrgb to CieXYZ            => None, BT_709_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(LinearSrgb to Bt2020            => None, BT_709_D65_TO_BT_2020_D65, None);
impl_conversion!(LinearSrgb to Oklab             => None, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(LinearSrgb to Oklch             => None, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(LinearSrgb to ICtCpPQ           => None, BT_709_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(LinearSrgb to EncodedAcesCgSrgb => None, BT_709_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(LinearSrgb to EncodedBt2020     => None, BT_709_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(LinearSrgb to EncodedBt2100PQ   => None, BT_709_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(LinearSrgb to EncodedDisplayP3  => None, BT_709_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(LinearSrgb to EncodedSrgb       => None, None, sRGB_oetf);

/// A type representing the [encoded sRGB][dynamic_spaces::ENCODED_SRGB] colorspace.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct EncodedSrgb;

impl_color_space! {
    EncodedSrgb is ENCODED_SRGB and Nonlinear,
    cint is EncodedSrgb,
    LinearSpace is LinearSrgb,
    Derefs as Rgb,
    Displays as "Encoded sRGB",
    Encodes from LinearSrgb,
}

#[cfg(not(target_arch = "spirv"))]
impl_as_u8_array!(EncodedSrgb: EncodedSrgb);

impl_conversion!(EncodedSrgb to EncodedSrgb => None, None, None);
impl_conversion!(EncodedSrgb to LinearSrgb => sRGB_eotf, None, None);
impl_conversion!(EncodedSrgb to AcesCg => sRGB_eotf, BT_709_D65_TO_AP1_D60, None);
impl_conversion!(EncodedSrgb to Aces2065 => sRGB_eotf, BT_709_D65_TO_AP0_D60, None);
impl_conversion!(EncodedSrgb to DisplayP3 => sRGB_eotf, BT_709_D65_TO_P3_D65, None);
impl_conversion!(EncodedSrgb to CieXYZ => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedSrgb to Bt2020 => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, None);
impl_conversion!(EncodedSrgb to Oklab => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedSrgb to Oklch => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedSrgb to ICtCpPQ => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedSrgb to EncodedAcesCgSrgb => sRGB_eotf, BT_709_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedSrgb to EncodedBt2020 => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedSrgb to EncodedBt2100PQ => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedSrgb to EncodedDisplayP3 => sRGB_eotf, BT_709_D65_TO_P3_D65, sRGB_oetf);

/// A type representing the reference [XYZ][dynamic_spaces::CIE_XYZ] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct CieXYZ;

impl_color_space! {
    CieXYZ is CIE_XYZ and Linear,
    cint is CieXYZ,
    Derefs as Xyz,
    Displays as "CIE XYZ",
}

impl_conversion!(CieXYZ to LinearSrgb        => None, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(CieXYZ to AcesCg            => None, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(CieXYZ to Aces2065          => None, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(CieXYZ to DisplayP3         => None, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(CieXYZ to CieXYZ            => None, None, None);
impl_conversion!(CieXYZ to Bt2020            => None, CIE_XYZ_D65_TO_BT_2020_D65, None);
impl_conversion!(CieXYZ to Oklab             => None, None, XYZ_to_Oklab);
impl_conversion!(CieXYZ to Oklch             => None, None, XYZ_to_Oklch);
impl_conversion!(CieXYZ to ICtCpPQ           => None, CIE_XYZ_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(CieXYZ to EncodedAcesCgSrgb => None, CIE_XYZ_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(CieXYZ to EncodedBt2020     => None, CIE_XYZ_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(CieXYZ to EncodedBt2100PQ   => None, CIE_XYZ_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(CieXYZ to EncodedDisplayP3  => None, CIE_XYZ_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(CieXYZ to EncodedSrgb       => None, CIE_XYZ_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [BT.2020][dynamic_spaces::BT_2020] color space
/// (equivalent to the linear BT.2100 color space).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Bt2020;

impl_color_space! {
    Bt2020 is BT_2020 and Linear,
    cint is Bt2020,
    Derefs as Rgb,
    Displays as "BT.2020",
}

impl_conversion!(Bt2020 to LinearSrgb        => None, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(Bt2020 to AcesCg            => None, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(Bt2020 to Aces2065          => None, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(Bt2020 to DisplayP3         => None, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(Bt2020 to CieXYZ            => None, BT_2020_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(Bt2020 to Bt2020            => None, None, None);
impl_conversion!(Bt2020 to Oklab             => None, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(Bt2020 to Oklch             => None, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(Bt2020 to ICtCpPQ           => None, None, RGB_to_ICtCp_PQ);
impl_conversion!(Bt2020 to EncodedAcesCgSrgb => None, BT_2020_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(Bt2020 to EncodedBt2020     => None, None, bt601_oetf);
impl_conversion!(Bt2020 to EncodedBt2100PQ   => None, None, ST_2084_PQ_eotf_inverse);
impl_conversion!(Bt2020 to EncodedDisplayP3  => None, BT_2020_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(Bt2020 to EncodedSrgb       => None, BT_2020_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the encoded [BT.2020][Bt2020] color space (with BT.2020 OETF applied).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct EncodedBt2020;

impl_color_space! {
    EncodedBt2020 is ENCODED_BT_2020 and Nonlinear,
    cint is EncodedBt2020,
    LinearSpace is Bt2020,
    Derefs as Rgb,
    Displays as "Encoded BT.2020",
    Encodes from Bt2020,
}

impl_conversion!(EncodedBt2020 to LinearSrgb        => bt601_oetf_inverse, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedBt2020 to AcesCg            => bt601_oetf_inverse, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(EncodedBt2020 to Aces2065          => bt601_oetf_inverse, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(EncodedBt2020 to DisplayP3         => bt601_oetf_inverse, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(EncodedBt2020 to CieXYZ            => bt601_oetf_inverse, BT_2020_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedBt2020 to Bt2020            => bt601_oetf_inverse, None, None);
impl_conversion!(EncodedBt2020 to Oklab             => bt601_oetf_inverse, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedBt2020 to Oklch             => bt601_oetf_inverse, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedBt2020 to ICtCpPQ           => bt601_oetf_inverse, None, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedBt2020 to EncodedAcesCgSrgb => bt601_oetf_inverse, BT_2020_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedBt2020 to EncodedBt2020     => None, None, None);
impl_conversion!(EncodedBt2020 to EncodedBt2100PQ   => bt601_oetf_inverse, None, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedBt2020 to EncodedDisplayP3  => bt601_oetf_inverse, BT_2020_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(EncodedBt2020 to EncodedSrgb       => bt601_oetf_inverse, BT_2020_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the BT.2100 color space (equivalent to the unencoded [BT.2020][Bt2020] color space).
pub type Bt2100 = Bt2020;

/// A type representing the encoded [BT.2100][Bt2100] color space (with inverse PQ EOTF applied).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct EncodedBt2100PQ;

impl_color_space! {
    EncodedBt2100PQ is ENCODED_BT_2100_PQ and Nonlinear,
    cint is EncodedBt2100PQ,
    LinearSpace is Bt2100,
    Derefs as Rgb,
    Displays as "Encoded BT.2100 (PQ)",
    Encodes from Bt2100,
}

impl_conversion!(EncodedBt2100PQ to LinearSrgb        => ST_2084_PQ_eotf, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedBt2100PQ to AcesCg            => ST_2084_PQ_eotf, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(EncodedBt2100PQ to Aces2065          => ST_2084_PQ_eotf, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(EncodedBt2100PQ to DisplayP3         => ST_2084_PQ_eotf, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(EncodedBt2100PQ to CieXYZ            => ST_2084_PQ_eotf, BT_2020_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedBt2100PQ to Bt2020            => ST_2084_PQ_eotf, None, None);
impl_conversion!(EncodedBt2100PQ to Oklab             => ST_2084_PQ_eotf, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedBt2100PQ to Oklch             => ST_2084_PQ_eotf, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedBt2100PQ to ICtCpPQ           => ST_2084_PQ_eotf, None, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedBt2100PQ to EncodedAcesCgSrgb => ST_2084_PQ_eotf, BT_2020_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedBt2100PQ to EncodedBt2020     => ST_2084_PQ_eotf, None, bt601_oetf);
impl_conversion!(EncodedBt2100PQ to EncodedBt2100PQ   => None, None, None);
impl_conversion!(EncodedBt2100PQ to EncodedDisplayP3  => ST_2084_PQ_eotf, BT_2020_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(EncodedBt2100PQ to EncodedSrgb       => ST_2084_PQ_eotf, BT_2020_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [ICtCp][dynamic_spaces::ICtCp_PQ] color space with PQ (Perceptual Quantizer) transfer functions.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct ICtCpPQ;

impl_color_space! {
    ICtCpPQ is ICtCp_PQ and Nonlinear,
    cint is ICtCpPQ,
    LinearSpace is CieXYZ,
    Derefs as ICtCp,
    Displays as "ICtCp (PQ)",
}

impl_conversion!(ICtCpPQ to LinearSrgb        => ICtCp_PQ_to_RGB, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(ICtCpPQ to AcesCg            => ICtCp_PQ_to_RGB, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(ICtCpPQ to Aces2065          => ICtCp_PQ_to_RGB, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(ICtCpPQ to DisplayP3         => ICtCp_PQ_to_RGB, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(ICtCpPQ to CieXYZ            => ICtCp_PQ_to_RGB, BT_2020_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(ICtCpPQ to Bt2020            => ICtCp_PQ_to_RGB, None, None);
impl_conversion!(ICtCpPQ to Oklab             => ICtCp_PQ_to_RGB, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(ICtCpPQ to Oklch             => ICtCp_PQ_to_RGB, BT_2020_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(ICtCpPQ to ICtCpPQ           => None, None, None);
impl_conversion!(ICtCpPQ to EncodedAcesCgSrgb => ICtCp_PQ_to_RGB, BT_2020_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(ICtCpPQ to EncodedBt2020     => ICtCp_PQ_to_RGB, None, bt601_oetf);
impl_conversion!(ICtCpPQ to EncodedBt2100PQ   => ICtCp_PQ_to_RGB, None, ST_2084_PQ_eotf_inverse);
impl_conversion!(ICtCpPQ to EncodedDisplayP3  => ICtCp_PQ_to_RGB, BT_2020_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(ICtCpPQ to EncodedSrgb       => ICtCp_PQ_to_RGB, BT_2020_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [Oklab][dynamic_spaces::OKLAB] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Oklab;

impl_color_space! {
    Oklab is OKLAB and Nonlinear,
    cint is Oklab,
    LinearSpace is CieXYZ,
    Derefs as Lab,
    Displays as "Oklab",
}

impl_conversion!(Oklab to LinearSrgb        => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(Oklab to AcesCg            => Oklab_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(Oklab to Aces2065          => Oklab_to_XYZ, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(Oklab to DisplayP3         => Oklab_to_XYZ, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(Oklab to CieXYZ            => Oklab_to_XYZ, None, None);
impl_conversion!(Oklab to Bt2020            => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, None);
impl_conversion!(Oklab to Oklab             => None, None, None);
impl_conversion!(Oklab to Oklch             => Oklab_to_XYZ, None, XYZ_to_Oklch);
impl_conversion!(Oklab to ICtCpPQ           => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(Oklab to EncodedAcesCgSrgb => Oklab_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(Oklab to EncodedBt2020     => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(Oklab to EncodedBt2100PQ   => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(Oklab to EncodedDisplayP3  => Oklab_to_XYZ, CIE_XYZ_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(Oklab to EncodedSrgb       => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [Oklch][dynamic_spaces::OKLCH] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Oklch;

impl_color_space! {
    Oklch is OKLCH and Nonlinear,
    cint is Oklch,
    LinearSpace is CieXYZ,
    Derefs as LCh,
    Displays as "Oklch",
}

impl_conversion!(Oklch to LinearSrgb        => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(Oklch to AcesCg            => Oklch_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(Oklch to Aces2065          => Oklch_to_XYZ, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(Oklch to DisplayP3         => Oklch_to_XYZ, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(Oklch to CieXYZ            => Oklch_to_XYZ, None, None);
impl_conversion!(Oklch to Bt2020            => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, None);
impl_conversion!(Oklch to Oklab             => Oklch_to_XYZ, None, XYZ_to_Oklab);
impl_conversion!(Oklch to Oklch             => None, None, None);
impl_conversion!(Oklch to ICtCpPQ           => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(Oklch to EncodedAcesCgSrgb => Oklch_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(Oklch to EncodedBt2020     => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(Oklch to EncodedBt2100PQ   => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(Oklch to EncodedDisplayP3  => Oklch_to_XYZ, CIE_XYZ_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(Oklch to EncodedSrgb       => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [ACEScg][dynamic_spaces::ACES_CG] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct AcesCg;

impl_color_space! {
    AcesCg is ACES_CG and Linear,
    cint is AcesCg,
    Derefs as Rgb,
    Displays as "ACEScg",
}

impl_conversion!(AcesCg to LinearSrgb        => None, AP1_D60_TO_BT_709_D65, None);
impl_conversion!(AcesCg to AcesCg            => None, None, None);
impl_conversion!(AcesCg to Aces2065          => None, AP1_D60_TO_AP0_D60, None);
impl_conversion!(AcesCg to DisplayP3         => None, AP1_D60_TO_P3_D65, None);
impl_conversion!(AcesCg to CieXYZ            => None, AP1_D60_TO_CIE_XYZ_D65, None);
impl_conversion!(AcesCg to Bt2020            => None, AP1_D60_TO_BT_2020_D65, None);
impl_conversion!(AcesCg to Oklab             => None, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(AcesCg to Oklch             => None, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(AcesCg to ICtCpPQ           => None, AP1_D60_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(AcesCg to EncodedAcesCgSrgb => None, None, sRGB_oetf);
impl_conversion!(AcesCg to EncodedBt2020     => None, AP1_D60_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(AcesCg to EncodedBt2100PQ   => None, AP1_D60_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(AcesCg to EncodedDisplayP3  => None, AP1_D60_TO_P3_D65, sRGB_oetf);
impl_conversion!(AcesCg to EncodedSrgb       => None, AP1_D60_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [ACEScg color space encoded with the sRGB transfer functions][dynamic_spaces::ENCODED_ACES_CG_SRGB].
///
/// This is useful to take advantage of many GPUs' hardware support for encoding and decoding using the
/// sRGB transfer functions. Using the sRGB transfer functions to encode ACEScg data is useful when trying to
/// use 8-bit texture formats. The OETF "compresses" the data to give better bit distribution based on
/// human color perception.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct EncodedAcesCgSrgb;

impl_color_space! {
    EncodedAcesCgSrgb is ENCODED_ACES_CG_SRGB and Nonlinear,
    LinearSpace is AcesCg,
    Derefs as Rgb,
    Displays as "Encoded ACEScg (sRGB)",
    Encodes from AcesCg,
}

impl_conversion!(EncodedAcesCgSrgb to LinearSrgb        => sRGB_eotf, AP1_D60_TO_BT_709_D65, None);
impl_conversion!(EncodedAcesCgSrgb to AcesCg            => sRGB_eotf, None, None);
impl_conversion!(EncodedAcesCgSrgb to Aces2065          => sRGB_eotf, AP1_D60_TO_AP0_D60, None);
impl_conversion!(EncodedAcesCgSrgb to DisplayP3         => sRGB_eotf, AP1_D60_TO_P3_D65, None);
impl_conversion!(EncodedAcesCgSrgb to CieXYZ            => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedAcesCgSrgb to Bt2020            => sRGB_eotf, AP1_D60_TO_BT_2020_D65, None);
impl_conversion!(EncodedAcesCgSrgb to Oklab             => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedAcesCgSrgb to Oklch             => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedAcesCgSrgb to ICtCpPQ           => sRGB_eotf, AP1_D60_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedAcesCgSrgb to EncodedAcesCgSrgb => None, None, None);
impl_conversion!(EncodedAcesCgSrgb to EncodedBt2020     => sRGB_eotf, AP1_D60_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedAcesCgSrgb to EncodedBt2100PQ   => sRGB_eotf, AP1_D60_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedAcesCgSrgb to EncodedDisplayP3  => sRGB_eotf, AP1_D60_TO_P3_D65, sRGB_oetf);
impl_conversion!(EncodedAcesCgSrgb to EncodedSrgb       => sRGB_eotf, AP1_D60_TO_BT_709_D65, sRGB_oetf);

impl AsU8 for EncodedAcesCgSrgb {}

/// A type representing the [ACES 2065-1][dynamic_spaces::ACES2065_1] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Aces2065;

impl_color_space! {
    Aces2065 is ACES2065_1 and Linear,
    cint is Aces2065,
    Derefs as Rgb,
    Displays as "ACES 2065-1",
}

impl_conversion!(Aces2065 to LinearSrgb        => None, AP0_D60_TO_BT_709_D65, None);
impl_conversion!(Aces2065 to AcesCg            => None, AP0_D60_TO_AP1_D60, None);
impl_conversion!(Aces2065 to Aces2065          => None, None, None);
impl_conversion!(Aces2065 to DisplayP3         => None, AP0_D60_TO_P3_D65, None);
impl_conversion!(Aces2065 to CieXYZ            => None, AP0_D60_TO_CIE_XYZ_D65, None);
impl_conversion!(Aces2065 to Bt2020            => None, AP0_D60_TO_BT_2020_D65, None);
impl_conversion!(Aces2065 to Oklab             => None, AP0_D60_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(Aces2065 to Oklch             => None, AP0_D60_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(Aces2065 to ICtCpPQ           => None, AP0_D60_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(Aces2065 to EncodedAcesCgSrgb => None, AP0_D60_TO_AP1_D60, sRGB_oetf);
impl_conversion!(Aces2065 to EncodedBt2020     => None, AP0_D60_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(Aces2065 to EncodedBt2100PQ   => None, AP0_D60_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(Aces2065 to EncodedDisplayP3  => None, AP0_D60_TO_P3_D65, sRGB_oetf);
impl_conversion!(Aces2065 to EncodedSrgb       => None, AP0_D60_TO_BT_709_D65, sRGB_oetf);

/// A type representing the Apple [Display P3][dynamic_spaces::DISPLAY_P3] color space.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DisplayP3;

impl_color_space! {
    DisplayP3 is DISPLAY_P3 and Linear,
    cint is DisplayP3,
    Derefs as Rgb,
    Displays as "Display P3",
}

impl_conversion!(DisplayP3 to LinearSrgb        => None, P3_D65_TO_BT_709_D65, None);
impl_conversion!(DisplayP3 to AcesCg            => None, P3_D65_TO_AP1_D60, None);
impl_conversion!(DisplayP3 to Aces2065          => None, P3_D65_TO_AP0_D60, None);
impl_conversion!(DisplayP3 to DisplayP3         => None, None, None);
impl_conversion!(DisplayP3 to CieXYZ            => None, P3_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(DisplayP3 to Bt2020            => None, P3_D65_TO_BT_2020_D65, None);
impl_conversion!(DisplayP3 to Oklab             => None, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(DisplayP3 to Oklch             => None, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(DisplayP3 to ICtCpPQ           => None, P3_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(DisplayP3 to EncodedAcesCgSrgb => None, P3_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(DisplayP3 to EncodedBt2020     => None, P3_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(DisplayP3 to EncodedBt2100PQ   => None, P3_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(DisplayP3 to EncodedDisplayP3  => None, None, sRGB_oetf);
impl_conversion!(DisplayP3 to EncodedSrgb       => None, P3_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the encoded [Display P3][DisplayP3] color space (with sRGB OETF applied).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct EncodedDisplayP3;

impl_color_space! {
    EncodedDisplayP3 is ENCODED_DISPLAY_P3 and Nonlinear,
    cint is EncodedDisplayP3,
    LinearSpace is DisplayP3,
    Derefs as Rgb,
    Displays as "Encoded Display P3",
    Encodes from DisplayP3,
}

impl_conversion!(EncodedDisplayP3 to LinearSrgb        => sRGB_eotf, P3_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedDisplayP3 to AcesCg            => sRGB_eotf, P3_D65_TO_AP1_D60, None);
impl_conversion!(EncodedDisplayP3 to Aces2065          => sRGB_eotf, P3_D65_TO_AP0_D60, None);
impl_conversion!(EncodedDisplayP3 to DisplayP3         => sRGB_eotf, None, None);
impl_conversion!(EncodedDisplayP3 to CieXYZ            => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedDisplayP3 to Bt2020            => sRGB_eotf, P3_D65_TO_BT_2020_D65, None);
impl_conversion!(EncodedDisplayP3 to Oklab             => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedDisplayP3 to Oklch             => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedDisplayP3 to ICtCpPQ           => sRGB_eotf, P3_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedDisplayP3 to EncodedAcesCgSrgb => sRGB_eotf, P3_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedDisplayP3 to EncodedBt2020     => sRGB_eotf, P3_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedDisplayP3 to EncodedBt2100PQ   => sRGB_eotf, P3_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedDisplayP3 to EncodedDisplayP3  => None, None, None);
impl_conversion!(EncodedDisplayP3 to EncodedSrgb       => sRGB_eotf, P3_D65_TO_BT_709_D65, sRGB_oetf);
