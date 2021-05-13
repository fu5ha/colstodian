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
        $space:ident is $dynamic_space:ident and Nonlinear,
        LinearSpace is $lin_space:ident,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
        Decodes to $decoded:ident via $decode_fn:ident, Encoded via $encode_fn:ident
    } => {
        impl_color_space! {
            $space is $dynamic_space and Nonlinear,
            LinearSpace is $lin_space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl DecodeFrom<$space> for $decoded {
            /// Decode the raw color from `$space` into `$decoded`
            fn decode_raw(color: Vec3) -> Vec3 {
                // White point useless here
                kolor::details::transform::$decode_fn(color, kolor::details::color::WhitePoint::A)
            }
        }

        impl EncodeFrom<$decoded> for $space {
            /// Encode the raw color from `$decoded` into `$space`
            fn encode_raw(color: Vec3) -> Vec3 {
                // White point useless here
                kolor::details::transform::$encode_fn(color, kolor::details::color::WhitePoint::A)
            }
        }
    };
    {
        $space:ident is $dynamic_space:ident and Linear,
        Derefs as $derefs_to:ident,
        Displays as $display:expr,
        $(Conversion to $dst_space:ident = $mat:ident),*
    } => {
        impl_color_space_inner! {
            $space is $dynamic_space,
            LinearSpace is $space,
            Derefs as $derefs_to,
            Displays as $display,
        }

        impl LinearColorSpace for $space {}

        $(impl LinearConvertFrom<$space> for $dst_space {
            const MATRIX: [f32; 9] = kolor::details::generated_matrices::$mat;
        })*
    };
}

use crate::component_structs::*;

/// A type representing the [linear sRGB][dynamic_spaces::LINEAR_SRGB] color space.
pub struct LinearSrgb;

impl_color_space! {
    LinearSrgb is LINEAR_SRGB and Linear,
    Derefs as Rgb,
    Displays as "Linear sRGB",
    Conversion to AcesCg = BT_709_D65_TO_AP1_D60,
    Conversion to Aces2065 = BT_709_D65_TO_AP0_D60,
    Conversion to DisplayP3 = BT_709_D65_TO_P3_D65,
    Conversion to CieXyz = BT_709_D65_TO_CIE_XYZ_D65,
    Conversion to Bt2020 = BT_709_D65_TO_BT_2020_D65
}

/// A type representing the [encoded sRGB][dynamic_spaces::ENCODED_SRGB] colorspace.
pub struct EncodedSrgb;

impl_color_space! {
    EncodedSrgb is ENCODED_SRGB and Nonlinear,
    LinearSpace is LinearSrgb,
    Derefs as Rgb,
    Displays as "Encoded sRGB",
    Decodes to LinearSrgb via sRGB_eotf, Encoded via sRGB_oetf
}

impl AsU8Array for EncodedSrgb {}

/// A type representing the reference [XYZ][dynamic_spaces::CIE_XYZ] color space.
pub struct CieXyz;

impl_color_space! {
    CieXyz is CIE_XYZ and Linear,
    Derefs as Xyz,
    Displays as "CIE XYZ",
    Conversion to AcesCg = CIE_XYZ_D65_TO_AP1_D60,
    Conversion to Aces2065 = CIE_XYZ_D65_TO_AP0_D60,
    Conversion to DisplayP3 = CIE_XYZ_D65_TO_P3_D65,
    Conversion to LinearSrgb = CIE_XYZ_D65_TO_BT_709_D65,
    Conversion to Bt2020 = CIE_XYZ_D65_TO_BT_2020_D65
}

/// A type representing the [BT.2020][dynamic_spaces::BT_2020] color space
/// (equivalent to the linear BT.2100 color space).
pub struct Bt2020;

impl_color_space! {
    Bt2020 is BT_2020 and Linear,
    Derefs as Rgb,
    Displays as "BT.2020",
    Conversion to LinearSrgb = BT_2020_D65_TO_BT_709_D65,
    Conversion to AcesCg = BT_2020_D65_TO_AP1_D60,
    Conversion to Aces2065 = BT_2020_D65_TO_AP0_D60,
    Conversion to CieXyz = BT_2020_D65_TO_CIE_XYZ_D65,
    Conversion to DisplayP3 = BT_2020_D65_TO_P3_D65
}

/// A type representing the encoded [BT.2020][Bt2020] color space (with BT.2020 OETF applied).
pub struct EncodedBt2020;

impl_color_space! {
    EncodedBt2020 is ENCODED_BT_2020 and Nonlinear,
    LinearSpace is Bt2020,
    Derefs as Rgb,
    Displays as "Encoded BT.2020",
    Decodes to Bt2020 via bt601_oetf_inverse, Encoded via bt601_oetf
}

/// A type representing the BT.2100 color space (equivalent to the unencoded [BT.2020][Bt2020] color space).
pub type Bt2100 = Bt2020;

/// A type representing the encoded [BT.2100][Bt2100] color space (with inverse PQ EOTF applied).
pub struct EncodedBt2100PQ;

impl_color_space! {
    EncodedBt2100PQ is ENCODED_BT_2100_PQ and Nonlinear,
    LinearSpace is Bt2020,
    Derefs as Rgb,
    Displays as "Encoded BT.2100 (PQ)",
    Decodes to Bt2020 via ST_2084_PQ_eotf, Encoded via ST_2084_PQ_eotf_inverse
}

/// A type representing the [ICtCp][dynamic_spaces::ICtCp_PQ] color space with PQ (Perceptual Quantizer) transfer functions.
pub struct ICtCpPQ;

impl_color_space! {
    ICtCpPQ is ICtCp_PQ and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as ICtCp,
    Displays as "ICtCp (PQ)",
}

/// A type representing the [Oklab][dynamic_spaces::OKLAB] color space.
pub struct Oklab;

impl_color_space! {
    Oklab is OKLAB and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as Lab,
    Displays as "Oklab",
}

/// A type representing the [Oklch][dynamic_spaces::OKLCH] color space.
pub struct Oklch;

impl_color_space! {
    Oklch is OKLCH and Nonlinear,
    LinearSpace is CieXyz,
    Derefs as LCh,
    Displays as "Oklch",
}

/// A type representing the [ACEScg][dynamic_spaces::ACES_CG] color space.
pub struct AcesCg;

impl_color_space! {
    AcesCg is ACES_CG and Linear,
    Derefs as Rgb,
    Displays as "ACEScg",
    Conversion to LinearSrgb = AP1_D60_TO_BT_709_D65,
    Conversion to Bt2020 = AP1_D60_TO_BT_2020_D65,
    Conversion to CieXyz = AP1_D60_TO_CIE_XYZ_D65,
    Conversion to Aces2065 = AP1_D60_TO_AP0_D60,
    Conversion to DisplayP3 = AP1_D60_TO_P3_D65
}

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
    Decodes to AcesCg via sRGB_eotf, Encoded via sRGB_oetf
}

impl AsU8Array for EncodedAcesCgSrgb {}

/// A type representing the [ACES 2065-1][dynamic_spaces::ACES2065_1] color space.
pub struct Aces2065;

impl_color_space! {
    Aces2065 is ACES2065_1 and Linear,
    Derefs as Rgb,
    Displays as "ACES 2065-1",
    Conversion to LinearSrgb = AP0_D60_TO_BT_709_D65,
    Conversion to Bt2020 = AP0_D60_TO_BT_2020_D65,
    Conversion to CieXyz = AP0_D60_TO_CIE_XYZ_D65,
    Conversion to AcesCg = AP0_D60_TO_AP1_D60,
    Conversion to DisplayP3 = AP0_D60_TO_P3_D65
}

/// A type representing the Apple [Display P3][dynamic_spaces::DISPLAY_P3] color space.
pub struct DisplayP3;

impl_color_space! {
    DisplayP3 is DISPLAY_P3 and Linear,
    Derefs as Rgb,
    Displays as "Display P3",
    Conversion to LinearSrgb = P3_D65_TO_BT_709_D65,
    Conversion to Bt2020 = P3_D65_TO_BT_2020_D65,
    Conversion to CieXyz = P3_D65_TO_CIE_XYZ_D65,
    Conversion to AcesCg = P3_D65_TO_AP1_D60,
    Conversion to Aces2065 = P3_D65_TO_AP0_D60
}

/// A type representing the encoded [Display P3][DisplayP3] color space (with sRGB OETF applied).
pub struct EncodedDisplayP3;

impl_color_space! {
    EncodedDisplayP3 is ENCODED_DISPLAY_P3 and Nonlinear,
    LinearSpace is DisplayP3,
    Derefs as Rgb,
    Displays as "Encoded Display P3",
    Decodes to DisplayP3 via sRGB_eotf, Encoded via sRGB_oetf
}
