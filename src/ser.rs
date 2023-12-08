use core::ops::Deref;

use crate::{
    Asyncness, Constness, Discriminant, Field, MaybeUninhabited, Safety, TypeLayout,
    TypeLayoutGraph, TypeLayoutInfo, TypeStructure, Variant,
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

pub const fn serialise_constness(bytes: &mut [u8], from: usize, value: Constness) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised Constness."
    );

    bytes[from] = match value {
        Constness::NonConst => b'r',
        Constness::Const => b'c',
    };

    from + 1
}

pub const fn serialised_constness_len(from: usize, _value: Constness) -> usize {
    from + 1
}

pub const fn serialise_asyncness(bytes: &mut [u8], from: usize, value: Asyncness) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised Asyncness."
    );

    bytes[from] = match value {
        Asyncness::Sync => b's',
        Asyncness::Async => b'a',
    };

    from + 1
}

pub const fn serialised_asyncness_len(from: usize, _value: Asyncness) -> usize {
    from + 1
}

pub const fn serialise_safety(bytes: &mut [u8], from: usize, value: Safety) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised Safety."
    );

    bytes[from] = match value {
        Safety::Safe => b's',
        Safety::Unsafe => b'u',
    };

    from + 1
}

pub const fn serialised_safety_len(from: usize, _value: Safety) -> usize {
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

pub const fn serialise_field(bytes: &mut [u8], from: usize, value: &Field) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_maybe_uninhabited(bytes, from, value.offset.map(()));
    let from = match value.offset {
        MaybeUninhabited::Inhabited(offset) => serialise_usize(bytes, from, offset),
        MaybeUninhabited::Uninhabited => from,
    };
    serialise_str(bytes, from, value.ty)
}

pub const fn serialised_field_len(from: usize, value: &Field) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_maybe_uninhabited_len(from, value.offset.map(()));
    let from = match value.offset {
        MaybeUninhabited::Inhabited(offset) => serialised_usize_len(from, offset),
        MaybeUninhabited::Uninhabited => from,
    };
    serialised_str_len(from, value.ty)
}

pub const fn serialise_fields(bytes: &mut [u8], from: usize, value: &[Field]) -> usize {
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

pub const fn serialise_variant<'a, F: ~const Deref<Target = [Field<'a>]>>(
    bytes: &mut [u8],
    from: usize,
    value: &Variant<'a, F>,
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_maybe_uninhabited(bytes, from, value.discriminant.map(()));
    let from = match &value.discriminant {
        MaybeUninhabited::Inhabited(discriminant) => {
            serialise_discriminant(bytes, from, discriminant)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialise_fields(bytes, from, &value.fields)
}

pub const fn serialised_variant_len<'a, F: ~const Deref<Target = [Field<'a>]>>(
    from: usize,
    value: &Variant<'a, F>,
) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_maybe_uninhabited_len(from, value.discriminant.map(()));
    let from = match &value.discriminant {
        MaybeUninhabited::Inhabited(discriminant) => {
            serialised_discriminant_len(from, discriminant)
        },
        MaybeUninhabited::Uninhabited => from,
    };
    serialised_fields_len(from, &value.fields)
}

pub const fn serialise_variants<'a, F: ~const Deref<Target = [Field<'a>]>>(
    bytes: &mut [u8],
    from: usize,
    value: &[Variant<'a, F>],
) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_variant(bytes, from, &value[i]);

        i += 1;
    }

    from
}

pub const fn serialised_variants_len<'a, F: ~const Deref<Target = [Field<'a>]>>(
    from: usize,
    value: &[Variant<'a, F>],
) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_variant_len(from, &value[i]);

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

pub const fn serialise_type_structure<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeStructure<'a, F, V, P>,
) -> usize {
    match value {
        TypeStructure::Primitive => serialise_byte(bytes, from, b'p'),
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
        TypeStructure::Function {
            constness,
            asyncness,
            safety,
            abi,
            parameters,
            r#return,
        } => {
            let from = serialise_byte(bytes, from, b'f');
            let from = serialise_constness(bytes, from, *constness);
            let from = serialise_asyncness(bytes, from, *asyncness);
            let from = serialise_safety(bytes, from, *safety);
            let from = serialise_str(bytes, from, abi);
            let from = serialise_parameters(bytes, from, parameters);
            serialise_str(bytes, from, r#return)
        },
    }
}

pub const fn serialised_type_structure_len<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
>(
    from: usize,
    value: &TypeStructure<'a, F, V, P>,
) -> usize {
    match value {
        TypeStructure::Primitive => serialised_byte_len(from, b'p'),
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
        TypeStructure::Function {
            constness,
            asyncness,
            safety,
            abi,
            parameters,
            r#return,
        } => {
            let from = serialised_byte_len(from, b'f');
            let from = serialised_constness_len(from, *constness);
            let from = serialised_asyncness_len(from, *asyncness);
            let from = serialised_safety_len(from, *safety);
            let from = serialised_str_len(from, abi);
            let from = serialised_parameters_len(from, parameters);
            serialised_str_len(from, r#return)
        },
    }
}

pub const fn serialise_type_layout_info<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo<'a, F, V, P>,
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.size);
    let from = serialise_usize(bytes, from, value.alignment);
    serialise_type_structure(bytes, from, &value.structure)
}

pub const fn serialised_type_layout_info_len<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
>(
    from: usize,
    value: &TypeLayoutInfo<'a, F, V, P>,
) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.size);
    let from = serialised_usize_len(from, value.alignment);
    serialised_type_structure_len(from, &value.structure)
}

const LAYOUT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const fn serialise_type_layout_graph<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
    I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
    G: ~const Deref<Target = [I]>,
>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph<'a, F, V, P, I, G>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialise_str(bytes, from, LAYOUT_VERSION);

    let from = serialise_str(bytes, from, value.ty);

    let mut from = serialise_usize(bytes, from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialise_type_layout_info(bytes, from, &*value.tys[i]);

        i += 1;
    }

    from
}

pub const fn serialised_type_layout_graph_len<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    P: ~const Deref<Target = [&'a str]>,
    I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
    G: ~const Deref<Target = [I]>,
>(
    from: usize,
    value: &TypeLayoutGraph<'a, F, V, P, I, G>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialised_str_len(from, LAYOUT_VERSION);

    let from = serialised_str_len(from, value.ty);

    let mut from = serialised_usize_len(from, value.tys.len());

    let mut i = 0;

    while i < value.tys.len() {
        from = serialised_type_layout_info_len(from, &*value.tys[i]);

        i += 1;
    }

    from
}
