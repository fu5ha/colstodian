pub use super::*;

/// Encodes that a color is [`Scene`]-referred (in the range [0..inf)), defined in relation
/// to input values from a (virtual) camera or other input device. Usually
/// representing something like radiance in an HDR rendering pipeline.
pub struct Scene;

impl State for Scene {
    const STATE: DynamicState = DynamicState::Scene;
}

/// Encodes that a color is [`Display`]-referred (in the range [0..1], defined
/// in relation to a display standard).
pub struct Display;

impl State for Display {
    const STATE: DynamicState = DynamicState::Display;
}
