use super::*;

/// Performs tonemapping on a given input color.
pub trait Tonemapper {
    /// Tonemap raw `color` using `self`
    fn tonemap_raw(&self, color: Vec3) -> Vec3;
}

/// Parameters for the [`LottesTonemapper`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LottesTonemaperParams {
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

impl Default for LottesTonemaperParams {
    fn default() -> Self {
        Self {
            contrast: 2.35,
            shoulder: 1.0,
            max_luminance: 150.0,
            gray_point_in: 0.18,
            gray_point_out: 0.18,
            crosstalk: 1.0,
            saturation: 1.0,
            cross_saturation: 1.2,
        }
    }
}

/// See this talk by Timothy Lottes https://gpuopen.com/wp-content/uploads/2016/03/GdcVdrLottes.pdf and associated slides
/// https://gpuopen.com/wp-content/uploads/2016/03/GdcVdrLottes.pdf
pub struct LottesTonemapper {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    crosstalk: f32,
    saturation: f32,
    cross_saturation: f32,
}

impl LottesTonemapper {
    /// Create a new [`LottesTonemapper`] with the given parameters.
    pub fn new(params: LottesTonemaperParams) -> Self {
        let LottesTonemaperParams {
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

    #[inline]
    fn tonemap_inner(&self, x: f32) -> f32 {
        let z = x.powf(self.a);
        z / (z.powf(self.d) * self.b + self.c)
    }
}

impl Tonemapper for LottesTonemapper {
    fn tonemap_raw(&self, color: Vec3) -> Vec3 {
        let max = color.max_element();
        let mut ratio = color / max;
        let tonemapped_max = self.tonemap_inner(max);

        ratio = ratio.powf(self.saturation / self.cross_saturation);
        ratio = ratio.lerp(Vec3::ONE, tonemapped_max.powf(self.crosstalk));
        ratio = ratio.powf(self.cross_saturation);

        (ratio * tonemapped_max).min(Vec3::ONE).max(Vec3::ZERO)
    }
}
