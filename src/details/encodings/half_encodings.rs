use glam::Vec3;
use half::f16;

use crate::component_structs::*;
use crate::linear_spaces;
use crate::reprs::*;
use crate::traits::*;

/// Three-component linearly-encoded "quasi-radiance" based on sRGB tristimulus primaries and white point in 16-bit
/// per component float.
/// 
/// See [`SrgbQuasiRadiance`][super::SrgbQuasiRadiance] for more information, this is the same as that but 16 bits
/// per component rather than 32.
pub struct SrgbQuasiRadianceF16;

impl ColorEncoding for SrgbQuasiRadianceF16 {
    type Repr = F16Repr;

    type ComponentStruct = Rgb<f16>;

    type LinearSpace = linear_spaces::Srgb;

    const NAME: &'static str = "SrgbQuasiRadianceF16";

    fn src_transform_raw(repr: Self::Repr) -> (glam::Vec3, f32) {
        let [r, g, b] = repr;
        (Vec3::new(r.to_f32(), g.to_f32(), b.to_f32()), 1.0)
    }

    fn dst_transform_raw(raw: glam::Vec3, _: f32) -> Self::Repr {
        [f16::from_f32(raw.x), f16::from_f32(raw.y), f16::from_f32(raw.z)]
    }
}

impl WorkingEncoding for SrgbQuasiRadianceF16 {}
impl QuasiRadianceEncoding for SrgbQuasiRadianceF16 {
    type BaseLinearSpace = linear_spaces::Srgb;
}
