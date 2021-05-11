use super::*;

pub use kolor::ColorSpace as DynamicColorSpace;

pub use dynamic_spaces::*;
/// Color spaces defined as data.
pub use kolor::spaces as dynamic_spaces;

/// Color spaces defined as types.
pub mod static_spaces {
    use super::*;

    macro_rules! impl_color_space {
        {
            $space:ident is $kolor_space:ident,
            Derefs as $derefs_to:ident,
        } => {
            impl<St> Deref for Color<$space, St> {
                type Target = $derefs_to;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    unsafe { &*(self as *const Self as *const Self::Target) }
                }
            }

            impl<St> DerefMut for Color<$space, St> {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *(self as *mut Self as *mut Self::Target) }
                }
            }

            impl ColorSpace for $space {
                const SPACE: DynamicColorSpace = kolor::spaces::$kolor_space;
            }
        };
        {
            $space:ident is $kolor_space:ident,
            Derefs as $derefs_to:ident,
            Decodes to $decoded:ident via $decode_fn:ident, Encoded via $encode_fn:ident
        } => {
            impl_color_space! {
                $space is $kolor_space,
                Derefs as $derefs_to,
            }

            impl DecodeFrom<$space> for $decoded {
                fn decode_raw(color: Vec3) -> Vec3 {
                    // White point useless here
                    kolor::details::transform::$decode_fn(color, kolor::details::color::WhitePoint::A)
                }
            }

            impl EncodeFrom<$decoded> for $space {
                fn encode_raw(color: Vec3) -> Vec3 {
                    // White point useless here
                    kolor::details::transform::$encode_fn(color, kolor::details::color::WhitePoint::A)
                }
            }
        };
        {
            $space:ident is $kolor_space:ident and Linear,
            Derefs as $derefs_to:ident,
            $(Conversion to $dst_space:ident = $mat:ident),*
        } => {
            impl_color_space! {
                $space is $kolor_space,
                Derefs as $derefs_to,
            }

            impl LinearColorSpace for $space {}

            $(impl LinearConvertFrom<$space> for $dst_space {
                const MATRIX: [f32; 9] = kolor::details::generated_matrices::$mat;
            })*
        };
    }

    use crate::component_structs::*;

    /// A type representing the linear (wihout OETF applied) sRGB color space.
    pub struct LinearSrgb;

    impl_color_space! {
        LinearSrgb is LINEAR_SRGB and Linear,
        Derefs as Rgb,
        Conversion to AcesCg = BT_709_D65_TO_AP0_D60,
        Conversion to Bt2020 = BT_709_D65_TO_BT_2020_D65
    }

    /// A type representing the encoded (with OETF applied) sRGB colorspace.
    pub struct EncodedSrgb;

    impl_color_space! {
        EncodedSrgb is ENCODED_SRGB,
        Derefs as Rgb,
        Decodes to LinearSrgb via sRGB_eotf, Encoded via sRGB_oetf
    }

    /// A type representing the BT.2020 color space (equivalent to the unencoded BT.2100 color space).
    ///
    /// See [BT_2020][dynamic_spaces::BT_2020] for more.
    pub struct Bt2020;

    impl_color_space! {
        Bt2020 is BT_2020 and Linear,
        Derefs as Rgb,
        Conversion to LinearSrgb = BT_2020_D65_TO_BT_709_D65,
        Conversion to AcesCg = BT_2020_D65_TO_AP0_D60,
        Conversion to DisplayP3 = BT_2020_D65_TO_P3_D65
    }

    /// A type representing the encoded [BT.2020][Bt2020] color space (with BT.2020 OETF applied).
    pub struct EncodedBt2020;

    impl_color_space! {
        EncodedBt2020 is ENCODED_BT_2020,
        Derefs as Rgb,
        Decodes to Bt2020 via bt601_oetf_inverse, Encoded via bt601_oetf
    }

    /// A type representing the BT.2100 color space (equivalent to the unencoded [BT.2020][Bt2020] color space).
    pub struct Bt2100;

    /// A type representing the encoded BT.2100 color space (with inverse PQ EOTF applied).
    pub struct EncodedBt2100PQ;

    impl_color_space! {
        EncodedBt2100PQ is ENCODED_BT_2100_PQ,
        Derefs as Rgb,
        Decodes to Bt2020 via ST_2084_PQ_eotf, Encoded via ST_2084_PQ_eotf_inverse
    }

    /// A type representing the ICtCp color space with PQ (Perceptual Quantizer) transfer functions.
    pub struct ICtCpPQ;

    impl_color_space! {
        ICtCpPQ is ICtCp_PQ,
        Derefs as ICtCp,
    }

    /// A type representing the ACEScg color space.
    pub struct AcesCg;

    impl_color_space! {
        AcesCg is ACES_CG and Linear,
        Derefs as Rgb,
        Conversion to LinearSrgb = AP0_D60_TO_BT_709_D65,
        Conversion to Bt2020 = AP0_D60_TO_BT_2020_D65,
        Conversion to DisplayP3 = AP0_D60_TO_P3_D65
    }

    /// A type representing the Apple Display P3 color space.
    pub struct DisplayP3;

    impl_color_space! {
        DisplayP3 is DISPLAY_P3 and Linear,
        Derefs as Rgb,
        Conversion to LinearSrgb = P3_D65_TO_BT_709_D65,
        Conversion to Bt2020 = P3_D65_TO_BT_2020_D65,
        Conversion to AcesCg = P3_D65_TO_AP0_D60
    }

    /// A type representing the encoded [Display P3][DisplayP3] color space (with sRGB OETF applied).
    pub struct EncodedDisplayP3;

    impl_color_space! {
        EncodedDisplayP3 is ENCODED_DISPLAY_P3,
        Derefs as Rgb,
        Decodes to DisplayP3 via sRGB_eotf, Encoded via sRGB_oetf
    }
}

pub use static_spaces::*;
