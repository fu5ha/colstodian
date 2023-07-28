The non-linear sRGB color encoding (with OETF applied) in 32 bit per component floats with separate alpha.

This is a moderately common way to specify color values.
If you have four floating point values from 0.0 to 1.0 which are directly analogous to
the 0-255 form (i.e. `(0.5, 0.5, 0.5, 0.5)` should be the same as `(127, 127, 127, 127)`), then this
is the color encoding you have. If you have the same kind of values but with no alpha component,
then you have [`SrgbF32`] instead.

Create a color in this encoding using [`Color::srgba_f32`].