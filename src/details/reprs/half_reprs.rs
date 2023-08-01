use super::*;

pub type F16Repr = [half::f16; 3];

impl ColorRepr for F16Repr {
    type Element = half::f16;
}
