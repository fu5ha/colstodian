The fully-encoded form of the sRGB color encoding standard.

This is one of the most common color encodings. If you have three u8 values (0-255)
or a hex code with 6 digits, this is almost certainly the encoding those values are encoded in.
If you have four u8 values (0-255) or a hex code with 8 digits, you likely have
a color in the [`SrgbAU8`] encoding instead.

Create a color in this encoding by using [`Color::srgb_u8`]