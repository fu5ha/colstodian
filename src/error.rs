use crate::{DynamicAlphaState, DynamicColorSpace, DynamicState};

use core::fmt;

/// Any error which can occur within the library.
#[derive(Debug)]
pub enum ColorError {
    DowncastFailed(DowncastError),
    DynamicConversionFailed(DynamicConversionError),
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ColorError::*;
        match self {
            DowncastFailed(src) => write!(
                f,
                "error when downcasting dynamic color to typed color: \n\\{}",
                src
            ),
            DynamicConversionFailed(src) => write!(
                f,
                "error when performing conversin on a dynamic color: \n\\{}",
                src
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ColorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ColorError::*;
        match self {
            DowncastFailed(src) => Some(src),
            DynamicConversionFailed(src) => Some(src),
        }
    }
}

impl From<DowncastError> for ColorError {
    fn from(err: DowncastError) -> Self {
        ColorError::DowncastFailed(err)
    }
}

impl From<DynamicConversionError> for ColorError {
    fn from(err: DynamicConversionError) -> Self {
        ColorError::DynamicConversionFailed(err)
    }
}

/// An error when downcasting from a dynamic color to a typed color.
#[derive(Debug)]
pub enum DowncastError {
    MismatchedSpace(DynamicColorSpace, DynamicColorSpace),
    MismatchedState(DynamicState, DynamicState),
    MismatchedAlphaState(DynamicAlphaState, DynamicAlphaState),
}

impl fmt::Display for DowncastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DowncastError::*;
        match self {
            MismatchedSpace(a, b) => write!(
                f,
                "color space did not match. actual: {:#?}, expected: {:#?}",
                a, b
            ),
            MismatchedState(a, b) => write!(
                f,
                "color state did not match. actual: {:#?}, expected: {:#?}",
                a, b
            ),
            MismatchedAlphaState(a, b) => write!(
                f,
                "color alpha state did not match. actual: {:#?}, expected: {:#?}",
                a, b
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DowncastError {}

/// An error that occurred when performing a conversion on a [`DynamicColor`][crate::DynamicColor]
#[derive(Debug)]
pub enum DynamicConversionError {
    NonlinearConversionInSceneState(DynamicColorSpace, DynamicColorSpace),
    StateChangeInNonlinearSpace(DynamicColorSpace, DynamicState, DynamicState),
    TonemapInDisplayState,
}

impl fmt::Display for DynamicConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DynamicConversionError::*;
        match self {
            NonlinearConversionInSceneState(a, b) => write!(f, "tried to convert from or to a nonlinear color space while in scene-referred state. current: {:#?}, requested: {:#?}", a, b),
            StateChangeInNonlinearSpace(a, b, c) => write!(f, "tried to change state while in a nonlinear color space. space: {:#?}, state: {:#?}, requested state: {:#?}", a, b, c),
            TonemapInDisplayState => write!(f, "attempted to tonemap in Display state"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DynamicConversionError {}

pub type ColorResult<T> = core::result::Result<T, ColorError>;
