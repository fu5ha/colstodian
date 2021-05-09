//! An opinionated color management library built on top of [`kolor`](https://docs.rs/kolor).
//!
//! # Introduction
//!
//! This library seeks to be a practical color management library for games and graphics.
//! It does so by encoding various information about a color either statically
//! in the Rust type system (as with the strongly-typed [`Color`]),
//! or data contained in the type (as with the dynamically-typed [`DynamicColor`]).
//!
//! Although it is designed to prevent footguns wherever possible, in order to make use of this library,
//! you should have a well-working understanding of basic color science and encoding principles.
//! As such, I highly recommend you give sections 2.0 and 2.1 of this document a read, as it is one
//! of the main inspirations for how this library is structured:
//!
//! ## Required Reading
//!
//! * http://github.com/jeremyselan/cinematiccolor/raw/master/ves/Cinematic_Color_VES.pdf
//!
//! # Overview
//!
//! `colstodian` is broken up into two 'halves', a statically-typed half which is meant to be
//! used as much as possible to help you prevent errors at compile time through leveraging the
//! Rust type system, and a dynamically-typed half which is meant to be used when serializing
//! and deserializing colors and otherwise interacting with colors from dynamic sources not
//! known at compile time.
//!
//! The core of the statically-typed half is the [`Color`] type, which encodes
//! two important pieces of metadata about the color in its type signature (`Color<Space, State>`): the
//! color's **color space** and **state**. If you read the required reading (you did, didn't you? ;))
//! then you should have a decent idea of what both of these things mean. To be clear, the **color space**
//! encodes the **primaries**, **white point**, and **transfer functions** upon which the color values are
//! based. The **state** encodes in which "direction" we relate the color values to real-world quantities:
//! either **scene-referred** or **display-referred**. Types which implement the [`ColorSpace`] and
//! [`State`] traits encode this information statically. Color spaces can be found in the `spaces` module
//! and states can be found in the `states` module.
//!
//! The core of the dynamically-typed half is the [`DynamicColor`] type, which encodes the color space
//! and state as data stored in the type at runtime. It stores these as [`DynamicColorSpace`]s and [`DynamicState`]s.

pub use kolor;
pub use kolor::ColorSpace as DynamicColorSpace;
pub use kolor::Mat3;
pub use kolor::Vec3;

use std::{marker::PhantomData, ops::*};

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use derivative::*;

pub mod spaces;
pub use spaces::*;

pub mod states;
pub use states::*;

pub mod tonemapper;
pub use tonemapper::*;

/// A strongly typed color, parameterized by a color space and state.
/// See [`ColorSpace`] and [`State`] for more.
#[derive(Derivative)]
#[derivative(Clone, Copy, PartialEq, Debug)]
pub struct Color<Spc, St> {
    /// The raw values of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec3,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Debug = "ignore")]
    _pd: PhantomData<(Spc, St)>,
}

impl<Spc, St> Color<Spc, St> {
    /// Creates a [`Color`] with the internal color elements `comp1`, `comp2`, `comp3`.
    pub fn new(el1: f32, el2: f32, el3: f32) -> Self {
        Self::from(Vec3::new(el1, el2, el3))
    }

    /// Creates a [`Color`] with raw values contained in `raw`.
    pub const fn from(raw: Vec3) -> Self {
        Self {
            raw,
            _pd: PhantomData,
        }
    }

    /// Clamp the raw element values of `self` in the range [0..1]
    pub fn saturate(self) -> Self {
        Self::from(self.raw.min(Vec3::ONE).max(Vec3::ZERO))
    }

    /// Get the maximum element of `self`
    pub fn max_element(self) -> f32 {
        self.raw.max_element()
    }
}

macro_rules! impl_op_color {
    ($op:ident, $op_func:ident) => {
        impl<Spc, St> $op for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from(self.raw.$op_func(rhs.raw))
            }
        }
    };
}

macro_rules! impl_op_color_float {
    ($op:ident, $op_func:ident) => {
        impl<Spc, St> $op<f32> for Color<Spc, St> {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: f32) -> Self::Output {
                Color::from(self.raw.$op_func(rhs))
            }
        }

        impl<Spc, St> $op<Color<Spc, St>> for f32 {
            type Output = Color<Spc, St>;
            fn $op_func(self, rhs: Color<Spc, St>) -> Self::Output {
                Color::from(self.$op_func(rhs.raw))
            }
        }
    };
}

impl_op_color!(Add, add);
impl_op_color!(Sub, sub);
impl_op_color!(Mul, mul);
impl_op_color!(Div, div);

impl_op_color_float!(Mul, mul);
impl_op_color_float!(Div, div);

/// A type that implements ColorSpace represents a specific color space. See the documentation
/// of [`DynamicColorSpace`] for more information about what a color space is.
pub trait ColorSpace {
    const SPACE: DynamicColorSpace;
}

/// Marks a type as representing a linear color space.
pub trait LinearColorSpace {}

/// A type that implements this trait is a color space for which a linear conversion
/// from `SrcSpace` to `Self` exists.
pub trait LinearConvertFrom<SrcSpc> {
    // TODO: use const Mat3s instead
    const MATRIX: [f32; 9];
}

/// A type that implements this trait represents a color's State.
///
/// All Colors have units. Sometimes a Color's units are explicit, such as measuring the emitted
/// light from a display using a spectroradiometer and being able to reference pixel values in CIE XYZ cd/m2.
/// Other times, the units are only indirectly related to the real world, and then providing a
/// mathematical conversion to measurable quantities. For example, in the case of display technology, common color encodings
/// (relations of code value to measurable XYZ performance) include sRGB, DCI-P3, and BT.2020.
///
/// Howver, considering color as a displayed quantity only provides part of the color encoding story. In addition to relating RGB
/// values to display measurements, one can also relate RGB values to the performance characteristics of an
/// *input device* (i.e., a camera, or a virtual camera in a 3d renderer). Input colorimetry can be measured in real world units as well.
/// In the case of a 3d renderer, these units are often (or at least should be) defined in the renderer as a radiometric quantity like
/// radiance, with the relation to XYZ values dictated by a linear transformation to the rendering color space.
/// Even in the case of a real world camera, it is not difficult to measure an input spectra with the spectrophotometer
/// in XYZ, and then compare this to the RGB values output from the camera.
///
/// It is a meaningful abstraction to categorize color spaces by the “direction” of this relationship to real world
/// quantities, which we refer to as State. Colors which are defined in relation to display
/// characteristic are called [`Display`]-referred, while color spaces which are defined in relation to input
/// devices (scenes) are [`Scene`]-referred.
pub trait State {
    const STATE: DynamicState;
}

impl<SrcSpace: ColorSpace, Display> Color<SrcSpace, Display> {
    /// Converts from one color space to another. This is only implemented in the generic case (for any ColorSpace)
    /// for Display-referred colors because non-linear color space transformations are often undefined for values
    /// outside the range [0..1].
    pub fn convert<DstSpace: ColorSpace>(self) -> Color<DstSpace, Display> {
        let conversion = kolor::ColorConversion::new(SrcSpace::SPACE, DstSpace::SPACE);
        Color::from(conversion.convert(self.raw))
    }
}

impl<SrcSpace, St> Color<SrcSpace, St> {
    /// Converts from a linear color space to another linear color space. This transformation ultimately
    /// boils down to a single 3x3 matrix * vector3 multiplication. This should be preferred when available
    /// over the more generic `Color::convert`.
    pub fn convert_linear<DstSpace: LinearConvertFrom<SrcSpace>>(self) -> Color<DstSpace, St> {
        let conversion_mat =
            Mat3::from_cols_array(&<DstSpace as LinearConvertFrom<SrcSpace>>::MATRIX).transpose();
        Color::from(conversion_mat * self.raw)
    }
}

impl<Spc: LinearColorSpace, SrcSt> Color<Spc, SrcSt> {
    /// Converts this color from one state to another. This conversion is usecase and even instance dependent.
    /// For example, converting a material's emissive texture value, a [`Display`]-referred color, to a [`Scene`]-referred
    /// color might take the form of a multiplication which scales the power of said emission into [`Scene`]-referred irradiance. On the other hand,
    /// converting a final [`Scene`]-referred color to a [`Display`]-referred color should be done with some kind of tonemapping
    /// operator. For a built-in, configurable tonemapper, see [`Tonemapper`].
    ///
    /// Note that the conversion function gives a raw color value, as the state of the color during the intermediate steps of the conversion
    /// is not really well defined. Therefore it's easier to just work on the raw values without type safety.
    pub fn convert_state<DstSt, F>(self, conversion_function: F) -> Color<Spc, DstSt>
    where
        F: FnOnce(Vec3) -> Vec3,
    {
        Color::from(conversion_function(self.raw))
    }

    /// Changes this color's State. This assumes that you have done some kind of conversion externally,
    /// or that the proper conversion is simply a noop.
    pub fn cast_state<DstSt>(self) -> Color<Spc, DstSt> {
        Color::from(self.raw)
    }
}

impl<Spc: LinearColorSpace> Color<Spc, Scene> {
    /// Tonemap `self` using the [`Tonemapper`] `tonemapper`, converting `self` from being
    /// scene-referred to being display-referred.
    pub fn tonemap(self, tonemapper: impl Tonemapper) -> Color<Spc, Display> {
        Color::from(tonemapper.tonemap_raw(self.raw))
    }
}

impl<Spc: ColorSpace, St: State> Color<Spc, St> {
    /// Upcasts `self` into a [`DynamicColor`]
    pub fn dynamic(self) -> DynamicColor {
        DynamicColor {
            raw: self.raw,
            space: Spc::SPACE,
            state: St::STATE,
        }
    }

    /// Checks whether the combination of color space and state types of `self` are valid.
    pub fn validate(self) -> Result<Self> {
        self.dynamic().validate()?;
        Ok(self)
    }
}

impl<Spc: ColorSpace, St: State> From<Color<Spc, St>> for DynamicColor {
    fn from(color: Color<Spc, St>) -> DynamicColor {
        color.dynamic()
    }
}

impl<Spc: ColorSpace, St: State> From<Color<Spc, St>> for kolor::Color {
    fn from(color: Color<Spc, St>) -> kolor::Color {
        kolor::Color {
            value: color.raw,
            space: Spc::SPACE,
        }
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

/// A dynamic color, with its Space and State defined
/// as data. This is mostly useful for (de)serialization.
/// See [`ColorSpace`] and [`State`] for more.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct DynamicColor {
    /// The raw tristimulus value of the color. Be careful when modifying this directly, i.e.
    /// don't multiply two Colors' raw values unless they are in the same color space and state.
    pub raw: Vec3,
    pub space: DynamicColorSpace,
    pub state: DynamicState,
}

/// Any error which can occur within the library.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error when downcasting dynamic color to typed color")]
    Downcast(#[from] DowncastError),
    #[error("error when performing conversion on a dynamic color")]
    DynamicConversionFailed(#[from] DynamicConversionError),
    #[error("error validating a color due to having both nonlinear color space and Scene state")]
    NonlinearSpaceAndSceneState,
}

/// An error when downcasting from a [`DynamicColor`] to a typed [`Color`].
#[derive(thiserror::Error, Debug)]
pub enum DowncastError {
    #[error("color space did not match. actual: {0:#?}, expected: {1:#?}")]
    MismatchedSpace(DynamicColorSpace, DynamicColorSpace),
    #[error("color state did not match. actual: {0:#?}, expected: {1:#?}")]
    MismatchedState(DynamicState, DynamicState),
}

/// An error that occurred when performing a conversion on a [`DynamicColor`]
#[derive(thiserror::Error, Debug)]
pub enum DynamicConversionError {
    #[error("tried to convert to or from a nonlinear color space while in scene-referred state. space: {0:#?}, requested space: {1:#?}")]
    NonlinearSpaceInSceneState(DynamicColorSpace, DynamicColorSpace),
    #[error("tried to change state while in a nonlinear color space. space: {0:#?}, state: {1:#?}, requested state: {2:#?}")]
    StateChangeInNonlinearSpace(DynamicColorSpace, DynamicState, DynamicState),
    #[error("attempted to tonemap in Display state")]
    TonemapInDisplayState,
}

pub type Result<T> = std::result::Result<T, Error>;

impl DynamicColor {
    pub fn new(raw: Vec3, space: DynamicColorSpace, state: DynamicState) -> Result<Self> {
        Self { raw, space, state }.validate()
    }

    /// Ensure that `self` has a valid combination of color space and state.
    pub fn validate(self) -> Result<Self> {
        if self.state == DynamicState::Scene && !self.space.is_linear() {
            Err(Error::NonlinearSpaceAndSceneState.into())
        } else {
            Ok(self)
        }
    }

    /// Attempt to convert to a typed `Color`. Returns an error if `self`'s color space and state do not match
    /// the given types.
    pub fn downcast<Spc: ColorSpace, St: State>(self) -> Result<Color<Spc, St>> {
        if self.space != Spc::SPACE {
            return Err(DowncastError::MismatchedSpace(self.space, Spc::SPACE).into());
        }

        if self.state != St::STATE {
            return Err(DowncastError::MismatchedState(self.state, St::STATE).into());
        }

        Ok(Color::from(self.raw))
    }

    /// Convert to a typed `Color` without checking if the color space and state types
    /// match this color's space and state. Use only if you are sure that this color
    /// is in the correct format.
    pub fn downcast_unchecked<Spc: ColorSpace, St: State>(self) -> Color<Spc, St> {
        Color::from(self.raw)
    }

    /// Convert `self` to the given color space. Must not attempt to convert to or from
    /// a nonlinear color space while in scene-referred state.
    pub fn convert(self, dest_space: DynamicColorSpace) -> Result<Self> {
        if self.state == DynamicState::Scene && (!self.space.is_linear() || !dest_space.is_linear())
        {
            return Err(
                DynamicConversionError::NonlinearSpaceInSceneState(self.space, dest_space).into(),
            );
        }

        let conversion = kolor::ColorConversion::new(self.space, dest_space);
        let raw = conversion.convert(self.raw);
        Ok(Self {
            raw,
            space: dest_space,
            state: self.state,
        })
    }

    /// Convert `self`'s state to the given state using the given conversion function.
    ///
    /// `self.space` must be linear. See docs for [`Color::<Space, State>::convert_state`]
    pub fn convert_state<F>(self, dest_state: DynamicState, conversion: F) -> Result<Self>
    where
        F: FnOnce(Vec3) -> Vec3,
    {
        if !self.space.is_linear() {
            return Err(DynamicConversionError::StateChangeInNonlinearSpace(
                self.space, self.state, dest_state,
            )
            .into());
        }
        Ok(Self {
            raw: conversion(self.raw),
            space: self.space,
            state: dest_state,
        })
    }

    /// Tonemap `self` using the [`Tonemapper`] `tonemapper`, converting `self` from being
    /// scene-referred to being display-referred.
    pub fn tonemap(mut self, tonemapper: impl Tonemapper) -> Result<Self> {
        if self.state != DynamicState::Scene {
            return Err(DynamicConversionError::TonemapInDisplayState.into());
        }
        if !self.space.is_linear() {
            return Err(DynamicConversionError::StateChangeInNonlinearSpace(
                self.space,
                self.state,
                DynamicState::Display,
            )
            .into());
        }
        self.raw = tonemapper.tonemap_raw(self.raw);
        Ok(self)
    }

    pub fn from_kolor(color: kolor::Color, state: DynamicState) -> Result<Self> {
        Self::new(color.value, color.space, state)
    }
}

impl From<DynamicColor> for kolor::Color {
    fn from(color: DynamicColor) -> kolor::Color {
        kolor::Color {
            value: color.raw,
            space: color.space,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let orig = Color::<EncodedSrgb, Display>::new(0.5, 0.5, 0.5);
        let col: Color<LinearSrgb, Display> = orig.convert();
        let col: Color<AcesCg, Display> = col.convert_linear();
        let col: Color<AcesCg, Scene> = col.convert_state(|c| c * 5.0);

        let col: Color<AcesCg, Display> = col.convert_state(|c| c / 5.0);
        let col: Color<LinearSrgb, Display> = col.convert();
        let fin: Color<EncodedSrgb, Display> = col.convert();

        assert_eq!(orig, fin);
    }
}
