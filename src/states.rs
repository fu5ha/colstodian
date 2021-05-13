use super::*;

use core::fmt;

/// Encodes that a color is [`Scene`]-referred (in the range [0..inf)), defined in relation
/// to input values from a (virtual) camera or other input device. Usually
/// representing something like radiance in an HDR rendering pipeline.
#[derive(Default)]
pub struct Scene;

impl State for Scene {
    const STATE: DynamicState = DynamicState::Scene;
}

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scene")
    }
}

/// Encodes that a color is [`Display`]-referred (in the range [0..1], defined
/// in relation to a display standard).
#[derive(Default)]
pub struct Display;

impl State for Display {
    const STATE: DynamicState = DynamicState::Display;
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Display")
    }
}

/// A dynamic version of a color's state. See docs for [`State`]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum DynamicState {
    /// See docs for [`Scene`]
    Scene,
    /// See docs for [`Display`]
    Display,
}
