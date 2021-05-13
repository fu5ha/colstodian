use super::*;

/// Any error which can occur within the library.
#[derive(thiserror::Error, Debug)]
pub enum ColorError {
    #[error("error when downcasting dynamic color to typed color")]
    Downcast(#[from] DowncastError),
    #[error("error when performing conversion on a dynamic color")]
    DynamicConversionFailed(#[from] DynamicConversionError),
}

/// An error when downcasting from a dynamic color to a typed color.
#[derive(thiserror::Error, Debug)]
pub enum DowncastError {
    #[error("color space did not match. actual: {0:#?}, expected: {1:#?}")]
    MismatchedSpace(DynamicColorSpace, DynamicColorSpace),
    #[error("color state did not match. actual: {0:#?}, expected: {1:#?}")]
    MismatchedState(DynamicState, DynamicState),
    #[error("color alpha state did not match. actual: {0:#?}, expected: {1:#?}")]
    MismatchedAlphaState(DynamicAlphaState, DynamicAlphaState),
}

/// An error that occurred when performing a conversion on a [`DynamicColor`]
#[derive(thiserror::Error, Debug)]
pub enum DynamicConversionError {
    #[error("tried to convert from or to a nonlinear color space while in scene-referred state. current: {0:#?}, requested: {1:#?}")]
    NonlinearConversionInSceneState(DynamicColorSpace, DynamicColorSpace),
    #[error("tried to change state while in a nonlinear color space. space: {0:#?}, state: {1:#?}, requested state: {2:#?}")]
    StateChangeInNonlinearSpace(DynamicColorSpace, DynamicState, DynamicState),
    #[error("attempted to tonemap in Display state")]
    TonemapInDisplayState,
    #[error("tried to convert from or to a nonlinear color space while in premultiplied alpha state. current: {0:#?}, requested: {1:#?}")]
    NonlinearConversionInPremultipliedAlphState(DynamicColorSpace, DynamicColorSpace),
}

pub type ColorResult<T> = std::result::Result<T, ColorError>;
