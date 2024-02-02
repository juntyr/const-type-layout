// use core::ops::Deref;
#![allow(clippy::needless_borrow, clippy::borrow_deref_ref)] // FIXME: Deref const trait

use core::num::NonZeroUsize;

use crate::{
    Discriminant, Field, MaybeUninhabited, TypeLayout, TypeLayoutGraph, TypeLayoutInfo,
    TypeStructure, Variant,
};

pub const fn serialise_str_literal(bytes: &mut [u8], from: usize, value: &str) -> usize {
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

// TODO: crate pub struct but used in public API
pub struct HashEntry<'a> {
    hash: u128,
    value: &'a str,
    index: usize,
    next: Option<usize>,
}

impl<'a> HashEntry<'a> {
    pub const EMPTY: Self = Self {
        hash: 0,
        value: "",
        index: usize::MAX,
        next: None,
    };

    pub const fn value(&self) -> &'a str {
        self.value
    }

    pub const fn next(&self) -> Option<usize> {
        self.next
    }
}

pub struct HashCursor {
    first: usize,
    last: usize,
    len: NonZeroUsize,
}

impl HashCursor {
    pub const fn first(&self) -> usize {
        self.first
    }
}

// adapted from rustc-hash, i.e. fxhash
const fn hash(a: &str) -> u128 {
    // (2^128 - 1) / pi, rounded to be odd
    const K: u128 = 0x517c_c1b7_2722_0a94_fe13_abe8_fa9a_6ee1;

    let mut a = a.as_bytes();

    let mut hash = 1_u128; // different from rustc-hash

    while let Some((a16, ar)) = a.split_first_chunk() {
        let value = u128::from_le_bytes(*a16);
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let Some((a8, ar)) = a.split_first_chunk() {
        let value = u64::from_le_bytes(*a8) as u128;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let Some((a4, ar)) = a.split_first_chunk() {
        let value = u32::from_le_bytes(*a4) as u128;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let Some((a2, ar)) = a.split_first_chunk() {
        let value = u16::from_le_bytes(*a2) as u128;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
        a = ar;
    }

    if let Some(a1) = a.first() {
        let value = *a1 as u128;
        hash = (hash.rotate_left(5) ^ value).wrapping_mul(K);
    }

    hash
}

const fn serialise_str<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &'a str,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let hash = hash(value);

    let mut i = (hash % (hashmap.len() as u128)) as usize;
    while (hashmap[i].index != usize::MAX)
        && ((hashmap[i].hash != hash) || !str_equal(hashmap[i].value, value))
    {
        i = (i + 1) % hashmap.len();
    }
    if hashmap[i].index == usize::MAX {
        hashmap[i].hash = hash;
        hashmap[i].value = value;
        if let Some(HashCursor {
            last: prev_index,
            len: prev_len,
            ..
        }) = cursor.as_mut()
        {
            hashmap[i].index = prev_len.get();
            hashmap[*prev_index].next = Some(i);
            *prev_index = i;
            *prev_len = prev_len.saturating_add(1);
        } else {
            hashmap[i].index = 0;
            *cursor = Some(HashCursor {
                first: i,
                last: i,
                len: NonZeroUsize::MIN,
            });
        }
        hashmap[i].next = None;
    }

    serialise_usize(bytes, from, hashmap[i].index)
}

const fn serialised_str_len(from: usize, value: &str, str_index: &mut usize) -> usize {
    // conservative estimate of the indices
    let from = serialised_usize_len(from, *str_index);
    *str_index += 1;

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

const fn serialise_byte(bytes: &mut [u8], from: usize, value: u8) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised byte."
    );

    bytes[from] = value;

    from + 1
}

const fn serialised_byte_len(from: usize, _value: u8) -> usize {
    from + 1
}

const fn serialise_maybe_uninhabited(
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

const fn serialised_maybe_uninhabited_len(from: usize, _value: MaybeUninhabited<()>) -> usize {
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

const fn serialise_discriminant(
    bytes: &mut [u8],
    from: usize,
    value: &Discriminant,
    hashmap: &mut [HashEntry],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let from = match value {
        Discriminant::I8(_) => serialise_str(
            bytes,
            from,
            <i8 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::I16(_) => serialise_str(
            bytes,
            from,
            <i16 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::I32(_) => serialise_str(
            bytes,
            from,
            <i32 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::I64(_) => serialise_str(
            bytes,
            from,
            <i64 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::I128(_) => serialise_str(
            bytes,
            from,
            <i128 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::Isize(_) => serialise_str(
            bytes,
            from,
            <isize as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::U8(_) => serialise_str(
            bytes,
            from,
            <u8 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::U16(_) => serialise_str(
            bytes,
            from,
            <u16 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::U32(_) => serialise_str(
            bytes,
            from,
            <u32 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::U64(_) => serialise_str(
            bytes,
            from,
            <u64 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::U128(_) => serialise_str(
            bytes,
            from,
            <u128 as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
        Discriminant::Usize(_) => serialise_str(
            bytes,
            from,
            <usize as TypeLayout>::TYPE_LAYOUT.ty.name(),
            hashmap,
            cursor,
        ),
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

const fn serialised_discriminant_len(
    from: usize,
    value: &Discriminant,
    str_index: &mut usize,
) -> usize {
    let from = match value {
        Discriminant::I8(_) => {
            serialised_str_len(from, <i8 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::I16(_) => {
            serialised_str_len(from, <i16 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::I32(_) => {
            serialised_str_len(from, <i32 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::I64(_) => {
            serialised_str_len(from, <i64 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::I128(_) => {
            serialised_str_len(from, <i128 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::Isize(_) => serialised_str_len(
            from,
            <isize as TypeLayout>::TYPE_LAYOUT.ty.name(),
            str_index,
        ),
        Discriminant::U8(_) => {
            serialised_str_len(from, <u8 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::U16(_) => {
            serialised_str_len(from, <u16 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::U32(_) => {
            serialised_str_len(from, <u32 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::U64(_) => {
            serialised_str_len(from, <u64 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::U128(_) => {
            serialised_str_len(from, <u128 as TypeLayout>::TYPE_LAYOUT.ty.name(), str_index)
        },
        Discriminant::Usize(_) => serialised_str_len(
            from,
            <usize as TypeLayout>::TYPE_LAYOUT.ty.name(),
            str_index,
        ),
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

const fn serialise_field<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &Field<'a>,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let from = serialise_str(bytes, from, value.name, hashmap, cursor);
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
    serialise_str(bytes, from, value.ty.name(), hashmap, cursor)
}

const fn str_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    // Safety: a and b are both valid strs and their lenght is equal
    //         compare_bytes is used since it has O(1) const eval cost
    unsafe { core::intrinsics::compare_bytes(a.as_ptr(), b.as_ptr(), a.len()) == 0 }
}

const fn serialised_field_len(from: usize, value: &Field, str_index: &mut usize) -> usize {
    let from = serialised_str_len(from, value.name, str_index);
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
    serialised_str_len(from, value.ty.name(), str_index)
}

const fn serialise_fields<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &[Field<'a>],
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_field(bytes, from, &value[i], hashmap, cursor);

        i += 1;
    }

    from
}

const fn serialised_fields_len(from: usize, value: &[Field], str_index: &mut usize) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_field_len(from, &value[i], str_index);

        i += 1;
    }

    from
}

const fn serialise_variant<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &Variant<'a>,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let from = serialise_str(bytes, from, value.name, hashmap, cursor);
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
            serialise_discriminant(bytes, from, discriminant, hashmap, cursor)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialise_fields(bytes, from, &value.fields, hashmap, cursor)
}

const fn serialised_variant_len(from: usize, value: &Variant, str_index: &mut usize) -> usize {
    let from = serialised_str_len(from, value.name, str_index);
    let from = serialised_maybe_uninhabited_len(
        from,
        match value.discriminant {
            MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(()),
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
        },
    );
    let from = match &value.discriminant {
        MaybeUninhabited::Inhabited(discriminant) => {
            serialised_discriminant_len(from, discriminant, str_index)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialised_fields_len(from, &value.fields, str_index)
}

const fn serialise_variants<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &[Variant<'a>],
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_variant(bytes, from, &value[i], hashmap, cursor);

        i += 1;
    }

    from
}

const fn serialised_variants_len(from: usize, value: &[Variant], str_index: &mut usize) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_variant_len(from, &value[i], str_index);

        i += 1;
    }

    from
}

const fn serialise_type_structure<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeStructure<'a>,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    match value {
        TypeStructure::Primitive => serialise_byte(bytes, from, b'p'),
        TypeStructure::Struct { repr, fields } => {
            let from = serialise_byte(bytes, from, b's');
            let from = serialise_str(bytes, from, repr, hashmap, cursor);
            serialise_fields(bytes, from, fields, hashmap, cursor)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialise_byte(bytes, from, b'u');
            let from = serialise_str(bytes, from, repr, hashmap, cursor);
            serialise_fields(bytes, from, fields, hashmap, cursor)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialise_byte(bytes, from, b'e');
            let from = serialise_str(bytes, from, repr, hashmap, cursor);
            serialise_variants(bytes, from, variants, hashmap, cursor)
        },
    }
}

const fn serialised_type_structure_len(
    from: usize,
    value: &TypeStructure,
    str_index: &mut usize,
) -> usize {
    match value {
        TypeStructure::Primitive => serialised_byte_len(from, b'p'),
        TypeStructure::Struct { repr, fields } => {
            let from = serialised_byte_len(from, b's');
            let from = serialised_str_len(from, repr, str_index);
            serialised_fields_len(from, fields, str_index)
        },
        TypeStructure::Union { repr, fields } => {
            let from = serialised_byte_len(from, b'u');
            let from = serialised_str_len(from, repr, str_index);
            serialised_fields_len(from, fields, str_index)
        },
        TypeStructure::Enum { repr, variants } => {
            let from = serialised_byte_len(from, b'e');
            let from = serialised_str_len(from, repr, str_index);
            serialised_variants_len(from, variants, str_index)
        },
    }
}

const fn serialise_type_layout_info<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo<'a>,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    let from = serialise_str(bytes, from, value.ty.name(), hashmap, cursor);
    let from = serialise_usize(bytes, from, value.size);
    let from = serialise_usize(bytes, from, value.alignment);
    serialise_type_structure(bytes, from, &value.structure, hashmap, cursor)
}

const fn serialised_type_layout_info_len(
    from: usize,
    value: &TypeLayoutInfo,
    str_index: &mut usize,
) -> usize {
    let from = serialised_str_len(from, value.ty.name(), str_index);
    let from = serialised_usize_len(from, value.size);
    let from = serialised_usize_len(from, value.alignment);
    serialised_type_structure_len(from, &value.structure, str_index)
}

const LAYOUT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const fn serialise_type_layout_graph<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph<'a>,
    hashmap: &mut [HashEntry<'a>],
    cursor: &mut Option<HashCursor>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialise_str(bytes, from, LAYOUT_VERSION, hashmap, cursor);

    let from = serialise_str(bytes, from, value.ty.name(), hashmap, cursor);

    let mut from = serialise_usize(bytes, from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialise_type_layout_info(bytes, from, &*value.tys[i], hashmap, cursor);

        i += 1;
    }

    from
}

pub const fn serialised_type_layout_graph_len(
    from: usize,
    value: &TypeLayoutGraph,
    str_index: &mut usize,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialised_str_len(from, LAYOUT_VERSION, str_index);

    let from = serialised_str_len(from, value.ty.name(), str_index);

    let mut from = serialised_usize_len(from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialised_type_layout_info_len(from, &*value.tys[i], str_index);

        i += 1;
    }

    from
}
