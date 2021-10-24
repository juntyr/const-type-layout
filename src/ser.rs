use crate::{Discriminant, Field, TypeLayoutGraph, TypeLayoutInfo, TypeStructure, Variant};

pub const fn serialise_str(bytes: &mut [u8], from: usize, value: &str) -> usize {
    let value_bytes = value.as_bytes();

    let from = serialise_usize(bytes, from, value_bytes.len());

    if (from + value_bytes.len()) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised str.");
    }

    let mut i = 0;

    while i < value_bytes.len() {
        bytes[from + i] = value_bytes[i];

        i += 1;
    }

    from + i
}

pub const fn serialised_str_len(from: usize, value: &str) -> usize {
    let value_bytes = value.as_bytes();

    let from = serialised_usize_len(from, value_bytes.len());

    from + value_bytes.len()
}

#[allow(clippy::cast_possible_truncation)]
pub const fn serialise_usize(bytes: &mut [u8], from: usize, value: usize) -> usize {
    if serialised_usize_len(from, value) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised usize.");
    }

    let mut rem = value;
    let mut i = 0;

    while rem > 0b0111_1111_usize {
        bytes[from + i] = ((rem & 0b0111_1111_usize) as u8) | 0b1000_0000_u8;

        i += 1;
        rem >>= 7;
    }

    bytes[from + i] = (rem & 0b0111_1111_usize) as u8;

    from + i + 1
}

pub const fn serialised_usize_len(from: usize, value: usize) -> usize {
    let mut rem = value;
    let mut i = 0;

    while rem > 127 {
        i += 1;
        rem >>= 7;
    }

    from + i + 1
}

pub const fn serialise_byte(bytes: &mut [u8], from: usize, value: u8) -> usize {
    if from >= bytes.len() {
        panic!("bytes is not large enough to contain the serialised byte.");
    }

    bytes[from] = value;

    from + 1
}

pub const fn serialised_byte_len(from: usize, _value: u8) -> usize {
    from + 1
}

pub const fn serialise_bool(bytes: &mut [u8], from: usize, value: bool) -> usize {
    if from >= bytes.len() {
        panic!("bytes is not large enough to contain the serialised bool.");
    }

    bytes[from] = if value { b'T' } else { b'F' };

    from + 1
}

pub const fn serialised_bool_len(from: usize, _value: bool) -> usize {
    from + 1
}

pub const fn serialise_discriminant<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &Discriminant<'a>,
) -> usize {
    let value_bytes = value.big_endian_bytes;

    let mut leading_zeroes = 0;

    while leading_zeroes < value_bytes.len() {
        if value_bytes[leading_zeroes] != 0_u8 {
            break;
        }

        leading_zeroes += 1;
    }

    let from = serialise_usize(bytes, from, value_bytes.len() - leading_zeroes);

    if (from + value_bytes.len() - leading_zeroes) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised discriminant.");
    }

    let mut i = leading_zeroes;

    while i < value_bytes.len() {
        bytes[from + i - leading_zeroes] = value_bytes[i];

        i += 1;
    }

    from + i - leading_zeroes
}

pub const fn serialised_discriminant_len(from: usize, value: &Discriminant) -> usize {
    let value_bytes = value.big_endian_bytes;

    let mut leading_zeroes = 0;

    while leading_zeroes < value_bytes.len() {
        if value_bytes[leading_zeroes] != 0_u8 {
            break;
        }

        leading_zeroes += 1;
    }

    let from = serialised_usize_len(from, value_bytes.len() - leading_zeroes);

    from + value_bytes.len() - leading_zeroes
}

pub const fn serialise_field<'a>(bytes: &mut [u8], from: usize, value: &Field<'a>) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.offset);
    serialise_str(bytes, from, value.ty)
}

pub const fn serialised_field_len(from: usize, value: &Field) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.offset);
    serialised_str_len(from, value.ty)
}

pub const fn serialise_fields<'a>(bytes: &mut [u8], from: usize, value: &[Field<'a>]) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_field(bytes, from, &value[i]);

        i += 1;
    }

    from
}

pub const fn serialised_fields_len(from: usize, value: &[Field]) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_field_len(from, &value[i]);

        i += 1;
    }

    from
}

pub const fn serialise_variant<'a>(bytes: &mut [u8], from: usize, value: &Variant<'a>) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_discriminant(bytes, from, &value.discriminant);
    serialise_fields(bytes, from, value.fields)
}

pub const fn serialised_variant_len(from: usize, value: &Variant) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_discriminant_len(from, &value.discriminant);
    serialised_fields_len(from, value.fields)
}

pub const fn serialise_variants<'a>(bytes: &mut [u8], from: usize, value: &[Variant<'a>]) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_variant(bytes, from, &value[i]);

        i += 1;
    }

    from
}

pub const fn serialised_variants_len(from: usize, value: &[Variant]) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_variant_len(from, &value[i]);

        i += 1;
    }

    from
}

pub const fn serialise_type_structure<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeStructure<'a>,
) -> usize {
    match value {
        TypeStructure::Struct { repr, fields } => {
            let from = serialise_byte(bytes, from, b's');
            let from = serialise_str(bytes, from, repr);
            serialise_fields(bytes, from, fields)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialise_byte(bytes, from, b'u');
            let from = serialise_str(bytes, from, repr);
            serialise_fields(bytes, from, fields)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialise_byte(bytes, from, b'e');
            let from = serialise_str(bytes, from, repr);
            serialise_variants(bytes, from, variants)
        },
        TypeStructure::Primitive => serialise_byte(bytes, from, b'v'),
        TypeStructure::Array { item, len } => {
            let from = serialise_byte(bytes, from, b'a');
            let from = serialise_str(bytes, from, item);
            serialise_usize(bytes, from, *len)
        },
        TypeStructure::Reference { inner, mutability } => {
            let from = serialise_byte(bytes, from, b'r');
            let from = serialise_str(bytes, from, inner);
            serialise_bool(bytes, from, *mutability)
        },
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialise_byte(bytes, from, b'p');
            let from = serialise_str(bytes, from, inner);
            serialise_bool(bytes, from, *mutability)
        },
    }
}

pub const fn serialised_type_structure_len(from: usize, value: &TypeStructure) -> usize {
    match value {
        TypeStructure::Struct { repr, fields } => {
            let from = serialised_byte_len(from, b's');
            let from = serialised_str_len(from, repr);
            serialised_fields_len(from, fields)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialised_byte_len(from, b'u');
            let from = serialised_str_len(from, repr);
            serialised_fields_len(from, fields)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialised_byte_len(from, b'e');
            let from = serialised_str_len(from, repr);
            serialised_variants_len(from, variants)
        },
        TypeStructure::Primitive => serialised_byte_len(from, b'v'),
        TypeStructure::Array { item, len } => {
            let from = serialised_byte_len(from, b'a');
            let from = serialised_str_len(from, item);
            serialised_usize_len(from, *len)
        },
        TypeStructure::Reference { inner, mutability } => {
            let from = serialised_byte_len(from, b'r');
            let from = serialised_str_len(from, inner);
            serialised_bool_len(from, *mutability)
        },
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialised_byte_len(from, b'p');
            let from = serialised_str_len(from, inner);
            serialised_bool_len(from, *mutability)
        },
    }
}

pub const fn serialise_type_layout_info<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo<'a>,
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.size);
    let from = serialise_usize(bytes, from, value.alignment);
    serialise_type_structure(bytes, from, &value.structure)
}

pub const fn serialised_type_layout_info_len(from: usize, value: &TypeLayoutInfo) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.size);
    let from = serialised_usize_len(from, value.alignment);
    serialised_type_structure_len(from, &value.structure)
}

const LAYOUT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const fn serialise_type_layout_graph<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph<'a>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialise_str(bytes, from, LAYOUT_VERSION);

    let from = serialise_str(bytes, from, value.ty);

    let mut from = serialise_usize(bytes, from, value.len);

    let mut i = 0;

    while i < value.len {
        from = serialise_type_layout_info(bytes, from, unsafe { &*value.tys[i] });

        i += 1;
    }

    from
}

pub const fn serialised_type_layout_graph_len(from: usize, value: &TypeLayoutGraph) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialised_str_len(from, LAYOUT_VERSION);

    let from = serialised_str_len(from, value.ty);

    let mut from = serialised_usize_len(from, value.len);

    let mut i = 0;

    while i < value.len {
        from = serialised_type_layout_info_len(from, unsafe { &*value.tys[i] });

        i += 1;
    }

    from
}
