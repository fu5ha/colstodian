use super::*;

use core::fmt;

/// Encodes that a color's component values have been premultiplied with its
/// alpha channel.
#[derive(Default)]
pub struct Premultiplied;

impl AlphaState for Premultiplied {
    const STATE: DynamicAlphaState = DynamicAlphaState::Premultiplied;
}

impl ConvertFromAlphaRaw<Separate> for Premultiplied {
    #[inline]
    fn convert_raw(raw: Vec3, alpha: f32) -> Vec3 {
        raw * alpha
    }
}

impl fmt::Display for Premultiplied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Premultiplied")
    }
}

/// Encodes that a color's component values are completely separate from its
/// alpha channel.
#[derive(Default)]
pub struct Separate;

impl AlphaState for Separate {
    const STATE: DynamicAlphaState = DynamicAlphaState::Separate;
}

impl ConvertFromAlphaRaw<Premultiplied> for Separate {
    #[inline]
    fn convert_raw(raw: Vec3, alpha: f32) -> Vec3 {
        if alpha != 0.0 {
            raw / alpha
        } else {
            raw
        }
    }
}

impl fmt::Display for Separate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Separate")
    }
}

/// A dynamic version of a color's alpha state. See docs for [`AlphaState`]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum DynamicAlphaState {
    /// See docs for [`Premultiplied`]
    Premultiplied,
    /// See docs for [`Separate`]
    Separate,
}
