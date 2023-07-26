use crate::traits::*;

pub type U8Repr = [u8; 3];

impl ColorRepr for U8Repr {
    type Element = u8;
}

pub type U8ARepr = [u8; 4];

impl ColorRepr for U8ARepr {
    type Element = u8;
}

pub type F32Repr = glam::Vec3;

impl ColorRepr for F32Repr {
    type Element = f32;
}

pub type F32ARepr = glam::Vec4;

impl ColorRepr for F32ARepr {
    type Element = f32;
}


