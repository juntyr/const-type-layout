#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Discriminant of an enum variant.
///
/// The [`Discriminant`] value can be constructed using the [`discriminant!`]
/// macro.
///
/// [`discriminant!`]: crate::discriminant!
pub struct Discriminant<'a> {
    /// The numeric value of the discriminant.
    ///
    /// The value is stored in [negabinary] representation in little-endian
    /// order as a byte array. Unlike the two's complement representation,
    /// [negabinary] has a unique representation for signed and unsigned
    /// numbers that is independent of the size of the value type (e.g. [`i16`]
    /// vs [`u64`]).
    ///
    /// Specifically,
    /// - the `(-2)^0`th digit is stored in the least significant bit of
    ///   `value[0]`
    /// - the `(-2)^7`th digit is stored in the most significant bit of
    ///   `value[0]`
    /// - the `(-2)^8`th digit is stored in the least significant bit of
    ///   `value[1]`
    /// - for all `i >= value.len()`, `value[i]` is implicitly all-zero
    ///
    /// [negabinary]: https://oeis.org/wiki/Negabinary
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub value: &'a [u8],
}

/// Helper macro to construct a [`Discriminant`] from its constant value.
///
/// The [`discriminant!`] macro is invoked with the constant expression value
/// of the discriminant, e.g.
///
/// [`discriminant!`]: crate::discriminant!
///
/// ```
/// # use const_type_layout::{discriminant, Discriminant};
/// // unsigned literal with inferred type
/// const D1: Discriminant = discriminant!(4);
///
/// // signed literal with inferred type
/// const D2: Discriminant = discriminant!(-2);
///
/// // unsigned literal with explicit type
/// const D3: Discriminant = discriminant!(2_u8);
///
/// // signed literal with explicit type
/// const D4: Discriminant = discriminant!(-4_i128);
///
/// // constant expression with inferred type
/// const D5: Discriminant = discriminant!(-4 + 7);
///
/// // constant value
/// const VALUE: isize = 42;
/// const D6: Discriminant = discriminant!(VALUE);
/// ```
#[macro_export]
macro_rules! discriminant {
    // https://oeis.org/wiki/Negabinary
    // https://en.wikipedia.org/wiki/Negative_base#To_negabinary
    ($x:expr) => {{
        const NBYTES: usize = {
            let mut x = $x;

            // number of bits required to represent x in negabinary
            let mut nbits = 0;

            // has the sign of the input been flipped?
            // since we cannot multiply by -1 if the input is unsigned, we keep
            //  track of sign flips instead
            let mut flipped_sign = false;

            while x != 0 {
                nbits += 1;

                // x.div_floor(2) without requring type annotations
                let (x_div_2, x_mod_2) = (x / 2, x % 2);
                let floor_x_2 = if x_mod_2 < 0 { x_div_2 - 1 } else { x_div_2 };

                // x := (x / -2) + ((x % -2) < 1)
                // where x's sign is flipped before or after
                if x_mod_2 == 0 {
                    x = floor_x_2;
                } else if flipped_sign {
                    x = floor_x_2 + 1;
                } else {
                    x = floor_x_2;
                }

                // dividing by -2 will flip the sign
                flipped_sign = !flipped_sign;
            }

            // round up to the number of bytes required for x in negabinary
            (nbits + 7) / 8
        };

        const NEGABINARY: [u8; NBYTES] = {
            let mut x = $x;

            // little endian byte array of the negabinary representation of x
            let mut bytes = [0; NBYTES];
            // current index into bytes
            let mut i = 0;
            // current bit mask for bytes[i]
            let mut bit = 0x1_u8;

            // has the sign of the input been flipped?
            // since we cannot multiply by -1 if the input is unsigned, we keep
            //  track of sign flips instead
            let mut flipped_sign = false;

            while x != 0 {
                // if x is odd, output a 1 to the bit array
                if (x % 2) != 0 {
                    bytes[i] |= bit;
                }

                // rotate the bit mask left, and increment i if it wraps around
                bit = bit.rotate_left(1);
                if bit == 0x1_u8 {
                    i += 1;
                }

                // x.div_floor(2) without requring type annotations
                let floor_x_2 = if (x % 2) < 0 { (x / 2) - 1 } else { x / 2 };

                // x := (x / -2) + ((x % -2) < 1)
                // where x's sign is flipped before or after
                if (x % 2) == 0 {
                    x = floor_x_2;
                } else if flipped_sign {
                    x = floor_x_2 + 1;
                } else {
                    x = floor_x_2;
                }

                // dividing by -2 will flip the sign
                flipped_sign = !flipped_sign;
            }

            bytes
        };

        // const-construct the discriminant
        $crate::Discriminant { value: &NEGABINARY }
    }};
}
