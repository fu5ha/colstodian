The fully-encoded form of the sRGB color encoding standard, with separate alpha component.

This is one of the most common color encodings. If you have four u8 values (0-255)
or a hex code with 8 digits, this is almost certainly the encoding those values are encoded in.
If you have three u8 values (0-255) or a hex code with 6 digits, you likely have
a color in the [`SrgbU8`] encoding instead.

Create a color in this encoding using [`Color::srgba_u8`].