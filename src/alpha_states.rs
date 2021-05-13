use super::*;

use core::fmt;

/// Encodes that a color's component values have been premultiplied with its
/// alpha channel.
#[derive(Default)]
pub struct Premultiplied;

impl AlphaState for Premultiplied {
    const STATE: DynamicAlphaState = DynamicAlphaState::Premultiplied;
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

impl fmt::Display for Separate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Separate")
    }
}

/// A dynamic version of a color's alpha state. See docs for [`Alpha`]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum DynamicAlphaState {
    /// See docs for [`Premultiplied`]
    Premultiplied,
    /// See docs for [`Separate`]
    Separate,
}
