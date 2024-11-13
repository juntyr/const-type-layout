#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Discriminant of an enum variant.
pub struct Discriminant<'a> {
    /// The numeric value of the discriminant.
    ///
    /// The value is stored in [negabinary] representation in little-endian
    /// order as a byte array.
    ///
    /// [negabinary]: https://oeis.org/wiki/Negabinary
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub value: &'a [u8],
}

#[macro_export]
#[doc(hidden)]
macro_rules! discriminant {
    // https://oeis.org/wiki/Negabinary
    // https://en.wikipedia.org/wiki/Negative_base#To_negabinary
    ($x:expr) => {{
        const NBYTES: usize = {
            let mut x = $x;
            let mut nbits = 0;

            let mut flipped_sign = false;

            while x != 0 {
                nbits += 1;

                let floor_x_2 = if (x % 2) < 0 { (x / 2) - 1 } else { x / 2 };

                if (x % 2) == 0 {
                    x = floor_x_2;
                } else if flipped_sign {
                    x = floor_x_2 + 1;
                } else {
                    x = floor_x_2;
                }

                flipped_sign = !flipped_sign;
            }

            ((nbits + 7) / 8) + 1
        };

        const NEGABINARY: [u8; NBYTES] = {
            let mut x = $x;
            let mut bits = [0; NBYTES];

            let mut i = 0;

            let mut bit = 0x1_u8;
            let mut flipped_sign = false;

            while x != 0 {
                if (x % 2) != 0 {
                    bits[i] |= bit;
                }
                bit = bit.rotate_left(1);

                if bit == 0x1_u8 {
                    i += 1;
                }

                let floor_x_2 = if (x % 2) < 0 { (x / 2) - 1 } else { x / 2 };

                if (x % 2) == 0 {
                    x = floor_x_2;
                } else if flipped_sign {
                    x = floor_x_2 + 1;
                } else {
                    x = floor_x_2;
                }

                flipped_sign = !flipped_sign;
            }

            bits
        };

        $crate::discriminant::Discriminant { value: &NEGABINARY }
    }};
}

#[doc(inline)]
pub use discriminant;
