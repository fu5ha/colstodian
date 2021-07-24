use crate::{
    spaces::{ICtCpPQ, LinearSrgb},
    Color, ColorInto, ColorSpace, Display, Scene,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use glam::{Vec3, Vec3Swizzles};
#[cfg(all(not(feature = "std"), feature = "libm"))]
use num_traits::Float;

/// Performs tonemapping on a given input color.
pub trait Tonemapper {
    type InputSpace: ColorSpace;
    type OutputSpace: ColorSpace;
    type Params;

    /// Tonemap `color` using `params`
    fn tonemap(
        color: impl ColorInto<Color<Self::InputSpace, Scene>>,
        params: Self::Params,
    ) -> Color<Self::OutputSpace, Display>;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct PerceptualTonemapperParams {
    /// The maximum desaturation for highlights. 0.0 is fully desaturated, 1.0 is no desaturation.
    pub desaturation: f32,
    /// The amount that colors should "shift" or "crosstalk" between channels, resulting in highlight desaturation.
    pub crosstalk: f32,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for PerceptualTonemapperParams {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for PerceptualTonemapperParams {}

impl Default for PerceptualTonemapperParams {
    fn default() -> Self {
        Self {
            desaturation: 0.1,
            crosstalk: 1.95,
        }
    }
}

/// A neutral, perceptual tonemapper based on tonemapping work in Frostbite as well as my own and
/// Tomasz Stachowiak's research on the subject. Should be able to be extended for HDR use in the
/// future.
pub struct PerceptualTonemapper;

impl PerceptualTonemapper {
    #[inline]
    fn tonemap_curve(v: f32) -> f32 {
        let c = v + v * v + 0.5 * v * v * v;
        c / (1.0 + c)
    }
}

impl Tonemapper for PerceptualTonemapper {
    type InputSpace = ICtCpPQ;
    type OutputSpace = ICtCpPQ;
    type Params = PerceptualTonemapperParams;

    fn tonemap(
        color: impl ColorInto<Color<Self::InputSpace, Scene>>,
        params: Self::Params,
    ) -> Color<Self::OutputSpace, Display> {
        let ictcp = color.into();

        let desat_amount = Self::tonemap_curve(ictcp.raw.yz().length() * 2.4);

        let intensity = ictcp.i;
        let display_rel_luminance = kolor::details::transform::pq::ST_2084_PQ_eotf_float(intensity);
        let tm_lum = Self::tonemap_curve(display_rel_luminance);
        let tm_intensity = kolor::details::transform::pq::ST_2084_PQ_eotf_inverse_float(tm_lum);

        let tm_col: Color<ICtCpPQ, Display> = Color::new(tm_intensity, ictcp.ct, ictcp.cp);

        let desat_col = tm_col.blend(
            Color::new(tm_intensity, 0.0, 0.0),
            desat_amount.powf(params.desaturation),
        );

        tm_col.blend(desat_col, tm_lum.clamp(0.0, 1.0).powf(params.crosstalk))
    }
}

/// Parameters for the [`LottesTonemapper`]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct LottesTonemapperParams {
    /// Controls the strength of the toe and shoulder rolloff
    pub contrast: f32,
    /// Controls the shape of the shoulder
    pub shoulder: f32,
    /// The maximum luminance of the input scene
    pub max_luminance: f32,
    /// Average 18% gray point of the input scene. Change this to control exposure.
    pub gray_point_in: f32,
    /// The average gray value of the output (ideally 0.18, modify this to control "brightness" slider in settings)
    pub gray_point_out: f32,
    /// Controls the amount of channel crosstalk
    pub crosstalk: f32,
    /// Controls saturation over the full tonal range
    pub saturation: f32,
    /// Controls saturation within channel crosstalk
    pub cross_saturation: f32,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for LottesTonemapperParams {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for LottesTonemapperParams {}

impl Default for LottesTonemapperParams {
    fn default() -> Self {
        Self {
            contrast: 2.35,
            shoulder: 1.0,
            max_luminance: 150.0,
            gray_point_in: 0.18,
            gray_point_out: 0.18,
            crosstalk: 10.0,
            saturation: 1.0,
            cross_saturation: 1.2,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BakedLottesTonemapperParams {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    crosstalk: f32,
    saturation: f32,
    cross_saturation: f32,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for BakedLottesTonemapperParams {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for BakedLottesTonemapperParams {}

impl From<LottesTonemapperParams> for BakedLottesTonemapperParams {
    fn from(params: LottesTonemapperParams) -> Self {
        let LottesTonemapperParams {
            contrast,
            shoulder,
            max_luminance,
            gray_point_in,
            gray_point_out,
            crosstalk,
            saturation,
            cross_saturation,
        } = params;

        let a = contrast;
        let d = shoulder;
        let gi_a = gray_point_in.powf(a);
        let gi_ad = gray_point_out.powf(a * d);
        let ml_a = max_luminance.powf(a);
        let ml_ad = max_luminance.powf(a * d);
        let denom_rcp = 1.0 / ((ml_ad - gi_ad) * gray_point_out);
        let b = (-gi_a + ml_a * gray_point_out) * denom_rcp;
        let c = (ml_ad * gi_a - ml_a * gi_ad * gray_point_out) * denom_rcp;

        Self {
            a,
            b,
            c,
            d,
            crosstalk,
            saturation,
            cross_saturation,
        }
    }
}

/// See this talk by Timothy Lottes <https://www.gdcvault.com/play/1023512/Advanced-Graphics-Techniques-Tutorial-Day>
/// and associated slides <https://gpuopen.com/wp-content/uploads/2016/03/GdcVdrLottes.pdf>
pub struct LottesTonemapper;

impl LottesTonemapper {
    #[inline]
    fn tonemap_inner(x: f32, params: BakedLottesTonemapperParams) -> f32 {
        let z = x.powf(params.a);
        z / (z.powf(params.d) * params.b + params.c)
    }
}

impl Tonemapper for LottesTonemapper {
    type InputSpace = LinearSrgb;
    type OutputSpace = LinearSrgb;
    type Params = BakedLottesTonemapperParams;

    fn tonemap(
        color: impl ColorInto<Color<Self::InputSpace, Scene>>,
        params: Self::Params,
    ) -> Color<Self::OutputSpace, Display> {
        let color = color.into();

        let max = color.raw.max_element();
        let mut ratio = color.raw / max;
        let tonemapped_max = Self::tonemap_inner(max, params);

        ratio = ratio.powf(params.saturation / params.cross_saturation);
        ratio = ratio.lerp(Vec3::ONE, tonemapped_max.powf(params.crosstalk));
        ratio = ratio.powf(params.cross_saturation);

        Color::from_raw((ratio * tonemapped_max).min(Vec3::ONE).max(Vec3::ZERO))
    }
}
