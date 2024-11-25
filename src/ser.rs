use crate::{
    Discriminant, Field, MaybeUninhabited, TypeLayoutGraph, TypeLayoutInfo, TypeStructure, Variant,
};

pub enum Serialiser<'a> {
    Hasher { state: u64 },
    Writer { buffer: &'a mut [u8], cursor: usize },
    Counter { cursor: usize },
}

impl<'a> Serialiser<'a> {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    #[must_use]
    pub const fn hasher(seed: u64) -> Self {
        let mut hasher = Self::Hasher {
            state: Self::FNV_OFFSET,
        };
        hasher.write_bytes(&seed.to_le_bytes());
        hasher
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        let Self::Hasher { state } = self else {
            panic!("not a hasher");
        };

        state
    }

    #[must_use]
    pub const fn writer(buffer: &'a mut [u8], cursor: usize) -> Self {
        Self::Writer { buffer, cursor }
    }

    #[must_use]
    pub const fn counter(cursor: usize) -> Self {
        Self::Counter { cursor }
    }

    #[must_use]
    pub const fn cursor(self) -> usize {
        let (Self::Counter { cursor } | Self::Writer { buffer: _, cursor }) = self else {
            panic!("not a writer or counter");
        };

        cursor
    }
}

impl Serialiser<'_> {
    #[inline]
    pub const fn write_bytes(&mut self, bytes: &[u8]) {
        match self {
            Self::Hasher { state } => {
                let mut i = 0;

                while i < bytes.len() {
                    *state ^= bytes[i] as u64;
                    *state = state.wrapping_mul(Self::FNV_PRIME);
                    i += 1;
                }
            },
            Self::Writer { buffer, cursor } => {
                assert!(
                    (*cursor + bytes.len()) <= buffer.len(),
                    "writer buffer exceeded"
                );

                let mut i = 0;

                while i < bytes.len() {
                    buffer[*cursor] = bytes[i];
                    *cursor += 1;
                    i += 1;
                }
            },
            Self::Counter { cursor: len } => *len += bytes.len(),
        }
    }

    #[inline]
    pub const fn write_byte(&mut self, byte: u8) {
        match self {
            Self::Hasher { state } => {
                *state ^= byte as u64;
                *state = state.wrapping_mul(Self::FNV_PRIME);
            },
            Self::Writer { buffer, cursor } => {
                assert!((*cursor + 1) <= buffer.len(), "writer buffer exceeded");

                buffer[*cursor] = byte;

                *cursor += 1;
            },
            Self::Counter { cursor: len } => *len += 1,
        }
    }
}

impl Serialiser<'_> {
    pub const fn serialise_str(&mut self, value: &str) {
        self.serialise_usize(value.len());
        self.write_bytes(value.as_bytes());
    }

    #[allow(clippy::cast_possible_truncation)]
    pub const fn serialise_usize(&mut self, value: usize) {
        let mut rem = value;

        while rem > 0b0111_1111_usize {
            self.write_byte(((rem & 0b0111_1111_usize) as u8) | 0b1000_0000_u8);
            rem >>= 7_u8;
        }

        self.write_byte((rem & 0b0111_1111_usize) as u8);
    }

    pub const fn serialise_byte(&mut self, value: u8) {
        self.write_byte(value);
    }

    pub const fn serialise_maybe_uninhabited(&mut self, value: MaybeUninhabited<()>) {
        self.write_byte(match value {
            MaybeUninhabited::Inhabited(()) => b'h',
            MaybeUninhabited::Uninhabited => b'n',
        });
    }

    const fn serialise_discriminant_bytes(&mut self, value_bytes: &[u8]) {
        let mut trailing_zeroes = 0;

        while trailing_zeroes < value_bytes.len() {
            if value_bytes[value_bytes.len() - 1 - trailing_zeroes] != 0_u8 {
                break;
            }

            trailing_zeroes += 1;
        }

        self.serialise_usize(value_bytes.len() - trailing_zeroes);

        let mut i = 0;

        while i < (value_bytes.len() - trailing_zeroes) {
            self.write_byte(value_bytes[i]);
            i += 1;
        }
    }

    pub const fn serialise_discriminant(&mut self, value: &Discriminant) {
        self.serialise_discriminant_bytes(value.value);
    }

    pub const fn serialise_field(&mut self, value: &Field) {
        self.serialise_str(value.name);
        self.serialise_maybe_uninhabited(match value.offset {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        });
        match value.offset {
            MaybeUninhabited::Inhabited(offset) => self.serialise_usize(offset),
            MaybeUninhabited::Uninhabited => (),
        };
        self.serialise_str(value.ty);
    }

    pub const fn serialise_fields(&mut self, value: &[Field]) {
        self.serialise_usize(value.len());

        let mut i = 0;

        while i < value.len() {
            self.serialise_field(&value[i]);

            i += 1;
        }
    }

    pub const fn serialise_variant(&mut self, value: &Variant) {
        self.serialise_str(value.name);
        self.serialise_maybe_uninhabited(match value.discriminant {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        });
        match &value.discriminant {
            MaybeUninhabited::Inhabited(discriminant) => {
                self.serialise_discriminant(discriminant);
            },
            MaybeUninhabited::Uninhabited => (),
        };
        self.serialise_fields(value.fields);
    }

    pub const fn serialise_variants(&mut self, value: &[Variant]) {
        self.serialise_usize(value.len());

        let mut i = 0;

        while i < value.len() {
            self.serialise_variant(&value[i]);

            i += 1;
        }
    }

    pub const fn serialise_parameters(&mut self, value: &[&str]) {
        self.serialise_usize(value.len());

        let mut i = 0;

        while i < value.len() {
            self.serialise_str(value[i]);

            i += 1;
        }
    }

    pub const fn serialise_type_structure(&mut self, value: &TypeStructure) {
        match value {
            TypeStructure::Primitive => self.serialise_byte(b'p'),
            TypeStructure::Struct { repr, fields } => {
                self.serialise_byte(b's');
                self.serialise_str(repr);
                self.serialise_fields(fields);
            },
            TypeStructure::Union { repr, fields } => {
                self.serialise_byte(b'u');
                self.serialise_str(repr);
                self.serialise_fields(fields);
            },
            TypeStructure::Enum { repr, variants } => {
                self.serialise_byte(b'e');
                self.serialise_str(repr);
                self.serialise_variants(variants);
            },
        }
    }

    pub const fn serialise_type_layout_info(&mut self, value: &TypeLayoutInfo) {
        self.serialise_str(value.name);
        self.serialise_usize(value.size);
        self.serialise_usize(value.alignment);
        self.serialise_type_structure(&value.structure);
    }

    pub const fn serialise_type_layout_graph(&mut self, value: &TypeLayoutGraph) {
        // Include the crate version of `type_layout` for cross-version comparison
        self.serialise_str(env!("CARGO_PKG_VERSION"));

        self.serialise_str(value.ty);

        self.serialise_usize(value.tys.len());

        let mut i = 0;

        while i < value.tys.len() {
            self.serialise_type_layout_info(&value.tys[i]);

            i += 1;
        }
    }
}
