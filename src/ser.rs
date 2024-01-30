// use core::ops::Deref;
#![allow(clippy::needless_borrow, clippy::borrow_deref_ref)] // FIXME: Deref const trait

use core::ops::BitXor;

use crate::{
    Discriminant, Field, MaybeUninhabited, TypeLayout, TypeLayoutGraph, TypeLayoutInfo,
    TypeStructure, Variant,
};

pub const fn serialise_str(bytes: &mut [u8], from: usize, value: &str) -> usize {
    let value_bytes = value.as_bytes();

    let from = serialise_usize(bytes, from, value_bytes.len());

    assert!(
        (from + value_bytes.len()) <= bytes.len(),
        "bytes is not large enough to contain the serialised str."
    );

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
    assert!(
        serialised_usize_len(from, value) <= bytes.len(),
        "bytes is not large enough to contain the serialised usize."
    );

    let mut rem = value;
    let mut i = 0;

    while rem > 0b0111_1111_usize {
        bytes[from + i] = ((rem & 0b0111_1111_usize) as u8) | 0b1000_0000_u8;

        i += 1;
        rem >>= 7_u8;
    }

    bytes[from + i] = (rem & 0b0111_1111_usize) as u8;

    from + i + 1
}

pub const fn serialised_usize_len(from: usize, value: usize) -> usize {
    let mut rem = value;
    let mut i = 0;

    while rem > 0b0111_1111_usize {
        i += 1;
        rem >>= 7_u8;
    }

    from + i + 1
}

#[allow(clippy::cast_possible_truncation)]
pub const fn serialise_index(bytes: &mut [u8], from: usize, value: usize, len: usize) -> usize {
    assert!(value < len, "index must be in range 0..len");
    assert!(
        serialised_index_len(from, len) <= bytes.len(),
        "bytes is not large enough to contain the serialised index."
    );

    let Some(mut max_rem) = len.checked_sub(1) else {
        return from;
    };
    let mut value_rem = value;

    let mut i = 0;

    while max_rem > 0b1111_1111_usize {
        bytes[from + i] = (value_rem & 0b1111_1111_usize) as u8;

        i += 1;
        max_rem >>= 8_u8;
        value_rem >>= 8_u8;
    }

    bytes[from + i] = (value_rem & 0b1111_1111_usize) as u8;

    from + i + 1
}

pub const fn serialised_index_len(from: usize, len: usize) -> usize {
    let Some(mut rem) = len.checked_sub(1) else {
        return from;
    };

    let mut i = 0;

    while rem > 0b1111_1111_usize {
        i += 1;
        rem >>= 8_u8;
    }

    from + i + 1
}

pub const fn serialise_byte(bytes: &mut [u8], from: usize, value: u8) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised byte."
    );

    bytes[from] = value;

    from + 1
}

pub const fn serialised_byte_len(from: usize, _value: u8) -> usize {
    from + 1
}

pub const fn serialise_maybe_uninhabited(
    bytes: &mut [u8],
    from: usize,
    value: MaybeUninhabited<()>,
) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised MaybeUninhabited."
    );

    bytes[from] = match value {
        MaybeUninhabited::Inhabited(()) => b'h',
        MaybeUninhabited::Uninhabited => b'n',
    };

    from + 1
}

pub const fn serialised_maybe_uninhabited_len(from: usize, _value: MaybeUninhabited<()>) -> usize {
    from + 1
}

const fn serialise_discriminant_bytes(bytes: &mut [u8], from: usize, value_bytes: &[u8]) -> usize {
    let mut leading_zeroes = 0;

    while leading_zeroes < value_bytes.len() {
        if value_bytes[leading_zeroes] != 0_u8 {
            break;
        }

        leading_zeroes += 1;
    }

    let from = serialise_usize(bytes, from, value_bytes.len() - leading_zeroes);

    assert!(
        (from + value_bytes.len() - leading_zeroes) <= bytes.len(),
        "bytes is not large enough to contain the serialised discriminant."
    );

    let mut i = leading_zeroes;

    while i < value_bytes.len() {
        bytes[from + i - leading_zeroes] = value_bytes[i];

        i += 1;
    }

    from + i - leading_zeroes
}

pub const fn serialise_discriminant(bytes: &mut [u8], from: usize, value: &Discriminant) -> usize {
    let from = match value {
        Discriminant::I8(_) => serialise_str(bytes, from, <i8 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I16(_) => serialise_str(bytes, from, <i16 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I32(_) => serialise_str(bytes, from, <i32 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I64(_) => serialise_str(bytes, from, <i64 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I128(_) => serialise_str(bytes, from, <i128 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::Isize(_) => {
            serialise_str(bytes, from, <isize as TypeLayout>::TYPE_LAYOUT.name)
        },
        Discriminant::U8(_) => serialise_str(bytes, from, <u8 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U16(_) => serialise_str(bytes, from, <u16 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U32(_) => serialise_str(bytes, from, <u32 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U64(_) => serialise_str(bytes, from, <u64 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U128(_) => serialise_str(bytes, from, <u128 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::Usize(_) => {
            serialise_str(bytes, from, <usize as TypeLayout>::TYPE_LAYOUT.name)
        },
    };

    match value {
        Discriminant::I8(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::I16(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::I32(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::I64(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::I128(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::Isize(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::U8(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::U16(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::U32(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::U64(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::U128(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
        Discriminant::Usize(v) => serialise_discriminant_bytes(bytes, from, &v.to_be_bytes()),
    }
}

const fn serialised_discriminant_bytes_len(from: usize, value_bytes: &[u8]) -> usize {
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

pub const fn serialised_discriminant_len(from: usize, value: &Discriminant) -> usize {
    let from = match value {
        Discriminant::I8(_) => serialised_str_len(from, <i8 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I16(_) => serialised_str_len(from, <i16 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I32(_) => serialised_str_len(from, <i32 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I64(_) => serialised_str_len(from, <i64 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::I128(_) => serialised_str_len(from, <i128 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::Isize(_) => serialised_str_len(from, <isize as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U8(_) => serialised_str_len(from, <u8 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U16(_) => serialised_str_len(from, <u16 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U32(_) => serialised_str_len(from, <u32 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U64(_) => serialised_str_len(from, <u64 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::U128(_) => serialised_str_len(from, <u128 as TypeLayout>::TYPE_LAYOUT.name),
        Discriminant::Usize(_) => serialised_str_len(from, <usize as TypeLayout>::TYPE_LAYOUT.name),
    };

    match value {
        Discriminant::I8(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::I16(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::I32(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::I64(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::I128(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::Isize(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::U8(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::U16(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::U32(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::U64(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::U128(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
        Discriminant::Usize(v) => serialised_discriminant_bytes_len(from, &v.to_be_bytes()),
    }
}

pub const fn serialise_field(
    bytes: &mut [u8],
    from: usize,
    value: &Field,
    tys: &[(&str, u64)],
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_maybe_uninhabited(
        bytes,
        from,
        match value.offset {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        },
    );
    let from = match value.offset {
        MaybeUninhabited::Inhabited(offset) => serialise_usize(bytes, from, offset),
        MaybeUninhabited::Uninhabited => from,
    };

    let ty = hash(value.ty);//const_fnv1a_hash::fnv1a_hash_str_32(value.ty);

    let mut i = 0;
    while i < tys.len() {
        if tys[i].1 == ty /* && str_equal(tys[i].0, value.ty) */ {
            break;
        }
        i += 1;
    }
    let ty_index = i;

    serialise_index(bytes, from, ty_index, tys.len())
}

pub const fn hash(a: &str) -> u64 {
    const K: u64 = 0x517c_c1b7_2722_0a95;

    let mut a = a.as_bytes();

    let mut hash = 1_u64; // different from rustc-hash

    while let [a0, a1, a2, a3, a4, a5, a6, a7, ar @ ..] = a {
        let value = u64::from_le_bytes([*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7]);
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let [a0, a1, a2, a3, ar @ ..] = a {
        let value = u32::from_le_bytes([*a0, *a1, *a2, *a3]) as u64;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let [a0, a1, ar @ ..] = a {
        let value = u16::from_le_bytes([*a0, *a1]) as u64;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let [a0, ..] = a {
        let value = *a0 as u64;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
    }

    hash
}

const fn str_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a = a.as_bytes();
    let b = b.as_bytes();

    let mut i = 0;

    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }

        i += 1;
    }

    true
}

pub const fn serialised_field_len(from: usize, value: &Field, _tys_len: usize) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_maybe_uninhabited_len(
        from,
        match value.offset {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        },
    );
    let from = match value.offset {
        MaybeUninhabited::Inhabited(offset) => serialised_usize_len(from, offset),
        MaybeUninhabited::Uninhabited => from,
    };
    // TODO: no longer truncate once comparisons have been made
    serialised_str_len(from, value.ty) // serialised_index_len(from, tys_len)
}

pub const fn serialise_fields(
    bytes: &mut [u8],
    from: usize,
    value: &[Field],
    tys: &[(&str, u64)],
) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_field(bytes, from, &value[i], tys);

        i += 1;
    }

    from
}

pub const fn serialised_fields_len(from: usize, value: &[Field], tys_len: usize) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_field_len(from, &value[i], tys_len);

        i += 1;
    }

    from
}

pub const fn serialise_variant(
    bytes: &mut [u8],
    from: usize,
    value: &Variant,
    tys: &[(&str, u64)],
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_maybe_uninhabited(
        bytes,
        from,
        match value.discriminant {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        },
    );
    let from = match &value.discriminant {
        MaybeUninhabited::Inhabited(discriminant) => {
            serialise_discriminant(bytes, from, discriminant)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialise_fields(bytes, from, &value.fields, tys)
}

pub const fn serialised_variant_len(from: usize, value: &Variant, tys_len: usize) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_maybe_uninhabited_len(
        from,
        match value.discriminant {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        },
    );
    let from = match &value.discriminant {
        MaybeUninhabited::Inhabited(discriminant) => {
            serialised_discriminant_len(from, discriminant)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialised_fields_len(from, &value.fields, tys_len)
}

pub const fn serialise_variants(
    bytes: &mut [u8],
    from: usize,
    value: &[Variant],
    tys: &[(&str, u64)],
) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_variant(bytes, from, &value[i], tys);

        i += 1;
    }

    from
}

pub const fn serialised_variants_len(from: usize, value: &[Variant], tys_len: usize) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_variant_len(from, &value[i], tys_len);

        i += 1;
    }

    from
}

pub const fn serialise_parameters(bytes: &mut [u8], from: usize, value: &[&str]) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_str(bytes, from, value[i]);

        i += 1;
    }

    from
}

pub const fn serialised_parameters_len(from: usize, value: &[&str]) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_str_len(from, value[i]);

        i += 1;
    }

    from
}

pub const fn serialise_type_structure(
    bytes: &mut [u8],
    from: usize,
    value: &TypeStructure,
    tys: &[(&str, u64)],
) -> usize {
    match value {
        TypeStructure::Primitive => serialise_byte(bytes, from, b'p'),
        TypeStructure::Struct { repr, fields } => {
            let from = serialise_byte(bytes, from, b's');
            let from = serialise_str(bytes, from, repr);
            serialise_fields(bytes, from, fields, tys)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialise_byte(bytes, from, b'u');
            let from = serialise_str(bytes, from, repr);
            serialise_fields(bytes, from, fields, tys)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialise_byte(bytes, from, b'e');
            let from = serialise_str(bytes, from, repr);
            serialise_variants(bytes, from, variants, tys)
        },
    }
}

pub const fn serialised_type_structure_len(
    from: usize,
    value: &TypeStructure,
    tys_len: usize,
) -> usize {
    match value {
        TypeStructure::Primitive => serialised_byte_len(from, b'p'),
        TypeStructure::Struct { repr, fields } => {
            let from = serialised_byte_len(from, b's');
            let from = serialised_str_len(from, repr);
            serialised_fields_len(from, fields, tys_len)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialised_byte_len(from, b'u');
            let from = serialised_str_len(from, repr);
            serialised_fields_len(from, fields, tys_len)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialised_byte_len(from, b'e');
            let from = serialised_str_len(from, repr);
            serialised_variants_len(from, variants, tys_len)
        },
    }
}

pub const fn serialise_type_layout_info(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo,
    tys: &[(&str, u64)],
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.size);
    let from = serialise_usize(bytes, from, value.alignment);
    serialise_type_structure(bytes, from, &value.structure, tys)
}

pub const fn serialised_type_layout_info_len(
    from: usize,
    value: &TypeLayoutInfo,
    tys_len: usize,
) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.size);
    let from = serialised_usize_len(from, value.alignment);
    serialised_type_structure_len(from, &value.structure, tys_len)
}

const LAYOUT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const fn serialise_type_layout_graph(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph,
    tys: &[(&str, u64)],
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialise_str(bytes, from, LAYOUT_VERSION);

    let from = serialise_str(bytes, from, value.ty);

    let mut from = serialise_usize(bytes, from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialise_type_layout_info(bytes, from, &*value.tys[i], tys);

        i += 1;
    }

    from
}

pub const fn serialised_type_layout_graph_len(
    from: usize,
    value: &TypeLayoutGraph,
    tys_len: usize,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialised_str_len(from, LAYOUT_VERSION);

    let from = serialised_str_len(from, value.ty);

    let mut from = serialised_usize_len(from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialised_type_layout_info_len(from, &*value.tys[i], tys_len);

        i += 1;
    }

    from
}
