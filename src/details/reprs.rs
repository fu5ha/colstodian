use crate::traits::*;

/// Just a `[u8; 3]`. Used for 8-bits-per-channel, three channel encodings.
pub type U8Repr = [u8; 3];

impl ColorRepr for U8Repr {
    type Element = u8;
}

/// Just a `[u8; 4]`. Used for 8-bits-per-channel, four channel encodings.
pub type U8ARepr = [u8; 4];

impl ColorRepr for U8ARepr {
    type Element = u8;
}

/// Just a [`glam::Vec3`] (also equivalent in layout to a `[f32; 3]`). Used for 32-bits-per-channel, three channel encodings.
pub type F32Repr = glam::Vec3;

impl ColorRepr for F32Repr {
    type Element = f32;
}

/// Just a [`glam::Vec4`] (also equivalent in layot to a `[f32; 4]`). Used for 32-bits-per-channel, four channel encodings.
pub type F32ARepr = glam::Vec4;

impl ColorRepr for F32ARepr {
    type Element = f32;
}
