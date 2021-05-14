use super::*;

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

macro_rules! impl_color_space_inner {
    {
        $space:ident is $dynamic_space:ident,
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl ColorSpace for $space {
            /// The [`DynamicColorSpace`] that this type represents.
            const SPACE: DynamicColorSpace = dynamic_spaces::$dynamic_space;

            /// The closest linear color space to this space.
            type LinearSpace = $lin_space;
        }

        impl Default for $space {
            fn default() -> Self {
                Self {}
            }
        }

        impl fmt::Display for $space {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $display)
            }
        }

        impl<St> Deref for Color<$space, St> {
            type Target = $derefs_to;

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

        impl<A> Deref for ColorAlpha<$space, A> {
            type Target = ColAlpha<$derefs_to>;

            /// Test
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(&self.raw as *const Vec4 as *const Self::Target) }
            }
        }

        impl<A> DerefMut for ColorAlpha<$space, A> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(&mut self.raw as *mut Vec4 as *mut Self::Target) }
            }
        }

    };
}

macro_rules! impl_color_space {
    {
        $space:ident is $dynamic_space:ident and Nonlinear,
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            LinearSpace is $lin_space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl NonlinearColorSpace for $space {}
    };
    {
        $space:ident is $dynamic_space:ident and Linear,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            LinearSpace is $space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl LinearColorSpace for $space {}
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
                let conversion_mat =
                    Mat3::from_cols_array(&kolor::details::generated_matrices::$mat).transpose();
                conversion_mat * color
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
                let conversion_mat =
                    Mat3::from_cols_array(&kolor::details::generated_matrices::$mat).transpose();
                conversion_mat * color
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
                let conversion_mat =
                    Mat3::from_cols_array(&kolor::details::generated_matrices::$mat).transpose();
                conversion_mat * color
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
                let conversion_mat =
                    Mat3::from_cols_array(&kolor::details::generated_matrices::$mat).transpose();
                conversion_mat * color
            }
            #[inline]
            fn dst_transform_raw(color: Vec3) -> Vec3 {
                kolor::details::transform::$dst_transform_fn(color, $dst_space::SPACE.white_point())
            }
        }
    };
}

/* Canonical conversion template
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
pub struct LinearSrgb;

impl_color_space! {
    LinearSrgb is LINEAR_SRGB and Linear,
    Derefs as Rgb,
    Displays as "Linear sRGB",
}

impl_conversion!(LinearSrgb to LinearSrgb        => None, None, None);
impl_conversion!(LinearSrgb to AcesCg            => None, BT_709_D65_TO_AP1_D60, None);
impl_conversion!(LinearSrgb to Aces2065          => None, BT_709_D65_TO_AP0_D60, None);
impl_conversion!(LinearSrgb to DisplayP3         => None, BT_709_D65_TO_P3_D65, None);
impl_conversion!(LinearSrgb to CieXyz            => None, BT_709_D65_TO_CIE_XYZ_D65, None);
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
pub struct EncodedSrgb;

impl_color_space! {
    EncodedSrgb is ENCODED_SRGB and Nonlinear,
    LinearSpace is LinearSrgb,
    Derefs as Rgb,
    Displays as "Encoded sRGB",
}

impl AsU8Array for EncodedSrgb {}

impl_conversion!(EncodedSrgb to LinearSrgb => sRGB_eotf, None, None);
impl_conversion!(EncodedSrgb to AcesCg => sRGB_eotf, BT_709_D65_TO_AP1_D60, None);
impl_conversion!(EncodedSrgb to Aces2065 => sRGB_eotf, BT_709_D65_TO_AP0_D60, None);
impl_conversion!(EncodedSrgb to DisplayP3 => sRGB_eotf, BT_709_D65_TO_P3_D65, None);
impl_conversion!(EncodedSrgb to CieXyz => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedSrgb to Bt2020 => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, None);
impl_conversion!(EncodedSrgb to Oklab => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedSrgb to Oklch => sRGB_eotf, BT_709_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedSrgb to ICtCpPQ => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedSrgb to EncodedAcesCgSrgb => sRGB_eotf, BT_709_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedSrgb to EncodedBt2020 => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedSrgb to EncodedBt2100PQ => sRGB_eotf, BT_709_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedSrgb to EncodedDisplayP3 => sRGB_eotf, BT_709_D65_TO_P3_D65, sRGB_oetf);

/// A type representing the reference [XYZ][dynamic_spaces::CIE_XYZ] color space.
pub struct CieXyz;

impl_color_space! {
    CieXyz is CIE_XYZ and Linear,
    Derefs as Xyz,
    Displays as "CIE XYZ",
}

impl_conversion!(CieXyz to LinearSrgb        => None, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(CieXyz to AcesCg            => None, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(CieXyz to Aces2065          => None, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(CieXyz to DisplayP3         => None, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(CieXyz to CieXyz            => None, None, None);
impl_conversion!(CieXyz to Bt2020            => None, CIE_XYZ_D65_TO_BT_2020_D65, None);
impl_conversion!(CieXyz to Oklab             => None, None, XYZ_to_Oklab);
impl_conversion!(CieXyz to Oklch             => None, None, XYZ_to_Oklch);
impl_conversion!(CieXyz to ICtCpPQ           => None, CIE_XYZ_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(CieXyz to EncodedAcesCgSrgb => None, CIE_XYZ_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(CieXyz to EncodedBt2020     => None, CIE_XYZ_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(CieXyz to EncodedBt2100PQ   => None, CIE_XYZ_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(CieXyz to EncodedDisplayP3  => None, CIE_XYZ_D65_TO_P3_D65, sRGB_oetf);
impl_conversion!(CieXyz to EncodedSrgb       => None, CIE_XYZ_D65_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [BT.2020][dynamic_spaces::BT_2020] color space
/// (equivalent to the linear BT.2100 color space).
pub struct Bt2020;

impl_color_space! {
    Bt2020 is BT_2020 and Linear,
    Derefs as Rgb,
    Displays as "BT.2020",
}

impl_conversion!(Bt2020 to LinearSrgb        => None, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(Bt2020 to AcesCg            => None, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(Bt2020 to Aces2065          => None, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(Bt2020 to DisplayP3         => None, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(Bt2020 to CieXyz            => None, BT_2020_D65_TO_CIE_XYZ_D65, None);
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
pub struct EncodedBt2020;

impl_color_space! {
    EncodedBt2020 is ENCODED_BT_2020 and Nonlinear,
    LinearSpace is Bt2020,
    Derefs as Rgb,
    Displays as "Encoded BT.2020",
}

impl_conversion!(EncodedBt2020 to LinearSrgb        => bt601_oetf_inverse, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedBt2020 to AcesCg            => bt601_oetf_inverse, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(EncodedBt2020 to Aces2065          => bt601_oetf_inverse, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(EncodedBt2020 to DisplayP3         => bt601_oetf_inverse, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(EncodedBt2020 to CieXyz            => bt601_oetf_inverse, BT_2020_D65_TO_CIE_XYZ_D65, None);
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
pub struct EncodedBt2100PQ;

impl_color_space! {
    EncodedBt2100PQ is ENCODED_BT_2100_PQ and Nonlinear,
    LinearSpace is Bt2020,
    Derefs as Rgb,
    Displays as "Encoded BT.2100 (PQ)",
}

impl_conversion!(EncodedBt2100PQ to LinearSrgb        => ST_2084_PQ_eotf, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedBt2100PQ to AcesCg            => ST_2084_PQ_eotf, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(EncodedBt2100PQ to Aces2065          => ST_2084_PQ_eotf, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(EncodedBt2100PQ to DisplayP3         => ST_2084_PQ_eotf, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(EncodedBt2100PQ to CieXyz            => ST_2084_PQ_eotf, BT_2020_D65_TO_CIE_XYZ_D65, None);
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
pub struct ICtCpPQ;

impl_color_space! {
    ICtCpPQ is ICtCp_PQ and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as ICtCp,
    Displays as "ICtCp (PQ)",
}

impl_conversion!(ICtCpPQ to LinearSrgb        => ICtCp_PQ_to_RGB, BT_2020_D65_TO_BT_709_D65, None);
impl_conversion!(ICtCpPQ to AcesCg            => ICtCp_PQ_to_RGB, BT_2020_D65_TO_AP1_D60, None);
impl_conversion!(ICtCpPQ to Aces2065          => ICtCp_PQ_to_RGB, BT_2020_D65_TO_AP0_D60, None);
impl_conversion!(ICtCpPQ to DisplayP3         => ICtCp_PQ_to_RGB, BT_2020_D65_TO_P3_D65, None);
impl_conversion!(ICtCpPQ to CieXyz            => ICtCp_PQ_to_RGB, BT_2020_D65_TO_CIE_XYZ_D65, None);
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
pub struct Oklab;

impl_color_space! {
    Oklab is OKLAB and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as Lab,
    Displays as "Oklab",
}

impl_conversion!(Oklab to LinearSrgb        => Oklab_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(Oklab to AcesCg            => Oklab_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(Oklab to Aces2065          => Oklab_to_XYZ, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(Oklab to DisplayP3         => Oklab_to_XYZ, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(Oklab to CieXyz            => Oklab_to_XYZ, None, None);
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
pub struct Oklch;

impl_color_space! {
    Oklch is OKLCH and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as LCh,
    Displays as "Oklch",
}

impl_conversion!(Oklch to LinearSrgb        => Oklch_to_XYZ, CIE_XYZ_D65_TO_BT_709_D65, None);
impl_conversion!(Oklch to AcesCg            => Oklch_to_XYZ, CIE_XYZ_D65_TO_AP1_D60, None);
impl_conversion!(Oklch to Aces2065          => Oklch_to_XYZ, CIE_XYZ_D65_TO_AP0_D60, None);
impl_conversion!(Oklch to DisplayP3         => Oklch_to_XYZ, CIE_XYZ_D65_TO_P3_D65, None);
impl_conversion!(Oklch to CieXyz            => Oklch_to_XYZ, None, None);
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
pub struct AcesCg;

impl_color_space! {
    AcesCg is ACES_CG and Linear,
    Derefs as Rgb,
    Displays as "ACEScg",
}

impl_conversion!(AcesCg to LinearSrgb        => None, AP1_D60_TO_BT_709_D65, None);
impl_conversion!(AcesCg to AcesCg            => None, None, None);
impl_conversion!(AcesCg to Aces2065          => None, AP1_D60_TO_AP0_D60, None);
impl_conversion!(AcesCg to DisplayP3         => None, AP1_D60_TO_P3_D65, None);
impl_conversion!(AcesCg to CieXyz            => None, AP1_D60_TO_CIE_XYZ_D65, None);
impl_conversion!(AcesCg to Bt2020            => None, AP1_D60_TO_BT_2020_D65, None);
impl_conversion!(AcesCg to Oklab             => None, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(AcesCg to Oklch             => None, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(AcesCg to ICtCpPQ           => None, AP1_D60_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(AcesCg to EncodedAcesCgSrgb => None, None, sRGB_oetf);
impl_conversion!(AcesCg to EncodedBt2020     => None, AP1_D60_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(AcesCg to EncodedBt2100PQ   => None, AP1_D60_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(AcesCg to EncodedDisplayP3  => None, AP1_D60_TO_P3_D65, sRGB_oetf);
impl_conversion!(AcesCg to EncodedSrgb       => None, AP1_D60_TO_BT_709_D65, sRGB_oetf);

/// A type representing the [ACEScg color space encoded with the sRGB transfer functions][dynamic_spaces::ENCODED_ACESCG_SRGB].
///
/// This is useful to take advantage of many GPUs' hardware support for encoding and decoding using the
/// sRGB transfer functions. Using the sRGB transfer functions to encode ACEScg data is useful when trying to
/// use 8-bit texture formats. The OETF "compresses" the data to give better bit distribution based on
/// human color perception.
pub struct EncodedAcesCgSrgb;

impl_color_space! {
    EncodedAcesCgSrgb is ENCODED_ACES_CG_SRGB and Nonlinear,
    LinearSpace is AcesCg,
    Derefs as Rgb,
    Displays as "Encoded ACEScg (sRGB)",
}

impl_conversion!(EncodedAcesCgSrgb to LinearSrgb        => sRGB_eotf, AP1_D60_TO_BT_709_D65, None);
impl_conversion!(EncodedAcesCgSrgb to AcesCg            => sRGB_eotf, None, None);
impl_conversion!(EncodedAcesCgSrgb to Aces2065          => sRGB_eotf, AP1_D60_TO_AP0_D60, None);
impl_conversion!(EncodedAcesCgSrgb to DisplayP3         => sRGB_eotf, AP1_D60_TO_P3_D65, None);
impl_conversion!(EncodedAcesCgSrgb to CieXyz            => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedAcesCgSrgb to Bt2020            => sRGB_eotf, AP1_D60_TO_BT_2020_D65, None);
impl_conversion!(EncodedAcesCgSrgb to Oklab             => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedAcesCgSrgb to Oklch             => sRGB_eotf, AP1_D60_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedAcesCgSrgb to ICtCpPQ           => sRGB_eotf, AP1_D60_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedAcesCgSrgb to EncodedAcesCgSrgb => None, None, None);
impl_conversion!(EncodedAcesCgSrgb to EncodedBt2020     => sRGB_eotf, AP1_D60_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedAcesCgSrgb to EncodedBt2100PQ   => sRGB_eotf, AP1_D60_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedAcesCgSrgb to EncodedDisplayP3  => sRGB_eotf, AP1_D60_TO_P3_D65, sRGB_oetf);
impl_conversion!(EncodedAcesCgSrgb to EncodedSrgb       => sRGB_eotf, AP1_D60_TO_BT_709_D65, sRGB_oetf);

impl AsU8Array for EncodedAcesCgSrgb {}

/// A type representing the [ACES 2065-1][dynamic_spaces::ACES2065_1] color space.
pub struct Aces2065;

impl_color_space! {
    Aces2065 is ACES2065_1 and Linear,
    Derefs as Rgb,
    Displays as "ACES 2065-1",
}

impl_conversion!(Aces2065 to LinearSrgb        => None, AP0_D60_TO_BT_709_D65, None);
impl_conversion!(Aces2065 to AcesCg            => None, AP0_D60_TO_AP1_D60, None);
impl_conversion!(Aces2065 to Aces2065          => None, None, None);
impl_conversion!(Aces2065 to DisplayP3         => None, AP0_D60_TO_P3_D65, None);
impl_conversion!(Aces2065 to CieXyz            => None, AP0_D60_TO_CIE_XYZ_D65, None);
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
pub struct DisplayP3;

impl_color_space! {
    DisplayP3 is DISPLAY_P3 and Linear,
    Derefs as Rgb,
    Displays as "Display P3",
}

impl_conversion!(DisplayP3 to LinearSrgb        => None, P3_D65_TO_BT_709_D65, None);
impl_conversion!(DisplayP3 to AcesCg            => None, P3_D65_TO_AP1_D60, None);
impl_conversion!(DisplayP3 to Aces2065          => None, P3_D65_TO_AP0_D60, None);
impl_conversion!(DisplayP3 to DisplayP3         => None, None, None);
impl_conversion!(DisplayP3 to CieXyz            => None, P3_D65_TO_CIE_XYZ_D65, None);
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
pub struct EncodedDisplayP3;

impl_color_space! {
    EncodedDisplayP3 is ENCODED_DISPLAY_P3 and Nonlinear,
    LinearSpace is DisplayP3,
    Derefs as Rgb,
    Displays as "Encoded Display P3",
}

impl_conversion!(EncodedDisplayP3 to LinearSrgb        => sRGB_eotf, P3_D65_TO_BT_709_D65, None);
impl_conversion!(EncodedDisplayP3 to AcesCg            => sRGB_eotf, P3_D65_TO_AP1_D60, None);
impl_conversion!(EncodedDisplayP3 to Aces2065          => sRGB_eotf, P3_D65_TO_AP0_D60, None);
impl_conversion!(EncodedDisplayP3 to DisplayP3         => sRGB_eotf, None, None);
impl_conversion!(EncodedDisplayP3 to CieXyz            => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, None);
impl_conversion!(EncodedDisplayP3 to Bt2020            => sRGB_eotf, P3_D65_TO_BT_2020_D65, None);
impl_conversion!(EncodedDisplayP3 to Oklab             => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklab);
impl_conversion!(EncodedDisplayP3 to Oklch             => sRGB_eotf, P3_D65_TO_CIE_XYZ_D65, XYZ_to_Oklch);
impl_conversion!(EncodedDisplayP3 to ICtCpPQ           => sRGB_eotf, P3_D65_TO_BT_2020_D65, RGB_to_ICtCp_PQ);
impl_conversion!(EncodedDisplayP3 to EncodedAcesCgSrgb => sRGB_eotf, P3_D65_TO_AP1_D60, sRGB_oetf);
impl_conversion!(EncodedDisplayP3 to EncodedBt2020     => sRGB_eotf, P3_D65_TO_BT_2020_D65, bt601_oetf);
impl_conversion!(EncodedDisplayP3 to EncodedBt2100PQ   => sRGB_eotf, P3_D65_TO_BT_2020_D65, ST_2084_PQ_eotf_inverse);
impl_conversion!(EncodedDisplayP3 to EncodedDisplayP3  => None, None, None);
impl_conversion!(EncodedDisplayP3 to EncodedSrgb       => sRGB_eotf, P3_D65_TO_BT_709_D65, sRGB_oetf);
