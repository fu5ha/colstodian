use crate::traits::*;

use glam::Vec3;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

impl<SrcSpace, SrcAlpha> ColorAlphaConversionQuery<SrcSpace, SrcAlpha> for Premultiplied
where
    SrcSpace: ConvertFromRaw<SrcSpace>,
    SrcAlpha: AlphaState,
    Self: ConvertFromAlphaRaw<SrcAlpha>,
{
    type DstSpace = SrcSpace;
    type DstAlpha = Self;
}

#[cfg(not(target_arch = "spirv"))]
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

impl<SrcSpace, SrcAlpha> ColorAlphaConversionQuery<SrcSpace, SrcAlpha> for Separate
where
    SrcSpace: ConvertFromRaw<SrcSpace>,
    SrcAlpha: AlphaState,
    Self: ConvertFromAlphaRaw<SrcAlpha>,
{
    type DstSpace = SrcSpace;
    type DstAlpha = Self;
}

#[cfg(not(target_arch = "spirv"))]
impl fmt::Display for Separate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Separate")
    }
}

/// A dynamic version of a color's alpha state. See docs for [`AlphaState`]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub enum DynamicAlphaState {
    /// See docs for [`Premultiplied`]
    Premultiplied,
    /// See docs for [`Separate`]
    Separate,
}
