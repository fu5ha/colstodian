use super::*;

pub use kolor::spaces as dynamic_spaces;

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

/// A bag of components with names r, g, b. Some `Color`s with RGB color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// A bag of components with names I, Ct, Cp. Some `Color`s with ICtCp color spaces
/// will `Deref`/`DerefMut` to this struct so that you can access their components with dot-syntax.
pub struct ICtCp {
    pub i: f32,
    pub ct: f32,
    pub cp: f32,
}

/// A type representing the linear (wihout OETF applied) sRGB color space.
pub struct LinearSrgb;

impl_color_space! {
    LinearSrgb is LINEAR_SRGB,
    Derefs as RGB,
    Conversion to AcesCg = BT_709_D65_TO_AP0_D60,
    Conversion to Bt2020 = BT_709_D65_TO_BT_2020_D65
}

/// A type representing the encoded (with OETF applied) sRGB colorspace.
pub struct EncodedSrgb;

impl_color_space! {
    EncodedSrgb is SRGB,
    Derefs as RGB,
}

/// A type representing the BT.2020 color space.
pub struct Bt2020;

impl_color_space! {
    Bt2020 is BT_2020,
    Derefs as RGB,
    Conversion to LinearSrgb = BT_2020_D65_TO_BT_709_D65,
    Conversion to AcesCg = BT_2020_D65_TO_AP0_D60
}

/// A type representing the ICtCp color space with PQ (Perceptual Quantizer) transfer functions.
pub struct ICtCpPq;

impl_color_space! {
    ICtCpPq is ICtCp_PQ,
    Derefs as ICtCp,
}

/// A type representing the ICtCp color space with HLG (hybrid log-gamma) transfer functions.
pub struct ICtCpHlg;

impl_color_space! {
    ICtCpHlg is ICtCp_HLG,
    Derefs as ICtCp,
}

/// A type representing the ACEScg color space.
pub struct AcesCg;

impl_color_space! {
    AcesCg is ACES_CG,
    Derefs as RGB,
    Conversion to LinearSrgb = AP0_D60_TO_BT_709_D65,
    Conversion to Bt2020 = AP0_D60_TO_BT_2020_D65
}
