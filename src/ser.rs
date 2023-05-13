use core::ops::Deref;

use crate::{
    Asyncness, Constness, Discriminant, Field, MaybeUninhabited, Mutability, Origin, Safety,
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

pub const fn serialise_mutability(bytes: &mut [u8], from: usize, value: Mutability) -> usize {
    assert!(
        from < bytes.len(),
        "bytes is not large enough to contain the serialised Mutability."
    );

    bytes[from] = match value {
        Mutability::Immutable => b'i',
        Mutability::Mutable => b'm',
    };

    from + 1
}

pub const fn serialised_mutability_len(from: usize, _value: Mutability) -> usize {
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

pub const fn serialise_origin(bytes: &mut [u8], from: usize, value: Origin) -> usize {
    assert!(
        serialised_origin_len(from, value) <= bytes.len(),
        "bytes is not large enough to contain the serialised Origin."
    );

    bytes[from] = match value {
        Origin::Internal => b'i',
        Origin::External { .. } => b'e',
    };

    match value {
        Origin::Internal => from + 1,
        Origin::External { abi } => serialise_str(bytes, from + 1, abi),
    }
}

pub const fn serialised_origin_len(from: usize, value: Origin) -> usize {
    match value {
        Origin::Internal => from + 1,
        Origin::External { abi } => serialised_str_len(from + 1, abi),
    }
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

pub const fn serialise_discriminant(bytes: &mut [u8], from: usize, value: &Discriminant) -> usize {
    let value_bytes = value.big_endian_bytes;

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
            serialise_mutability(bytes, from, *mutability)
        },
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialise_byte(bytes, from, b'p');
            let from = serialise_str(bytes, from, inner);
            serialise_mutability(bytes, from, *mutability)
        },
        TypeStructure::FunctionItem {
            constness,
            asyncness,
            safety,
            origin,
            parameters,
            r#return,
        } => {
            let from = serialise_byte(bytes, from, b'i');
            let from = serialise_constness(bytes, from, *constness);
            let from = serialise_asyncness(bytes, from, *asyncness);
            let from = serialise_safety(bytes, from, *safety);
            let from = serialise_origin(bytes, from, *origin);
            let from = serialise_parameters(bytes, from, parameters);
            serialise_str(bytes, from, r#return)
        },
        TypeStructure::FunctionPointer {
            safety,
            origin,
            parameters,
            r#return,
        } => {
            let from = serialise_byte(bytes, from, b'f');
            let from = serialise_safety(bytes, from, *safety);
            let from = serialise_origin(bytes, from, *origin);
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
            serialised_mutability_len(from, *mutability)
        },
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialised_byte_len(from, b'p');
            let from = serialised_str_len(from, inner);
            serialised_mutability_len(from, *mutability)
        },
        TypeStructure::FunctionItem {
            constness,
            asyncness,
            safety,
            origin,
            parameters,
            r#return,
        } => {
            let from = serialised_byte_len(from, b'i');
            let from = serialised_constness_len(from, *constness);
            let from = serialised_asyncness_len(from, *asyncness);
            let from = serialised_safety_len(from, *safety);
            let from = serialised_origin_len(from, *origin);
            let from = serialised_parameters_len(from, parameters);
            serialised_str_len(from, r#return)
        },
        TypeStructure::FunctionPointer {
            safety,
            origin,
            parameters,
            r#return,
        } => {
            let from = serialised_byte_len(from, b'f');
            let from = serialised_safety_len(from, *safety);
            let from = serialised_origin_len(from, *origin);
            let from = serialised_parameters_len(from, parameters);
            serialised_str_len(from, r#return)
        },
    }
}

pub const fn serialise_type_layout_info<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo<'a, F, V>,
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
>(
    from: usize,
    value: &TypeLayoutInfo<'a, F, V>,
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
    I: ~const Deref<Target = TypeLayoutInfo<'a, F, V>>,
>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph<'a, F, V, I>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialise_str(bytes, from, LAYOUT_VERSION);

    let from = serialise_str(bytes, from, value.ty);

    let mut len = 0;

    while len < value.tys.len() && value.tys[len].is_some() {
        len += 1;
    }

    let mut from = serialise_usize(bytes, from, len);

    let mut i = 0;

    while i < len {
        if let Some(ty) = &value.tys[i] {
            from = serialise_type_layout_info(bytes, from, ty);
        }

        i += 1;
    }

    from
}

pub const fn serialised_type_layout_graph_len<
    'a,
    F: ~const Deref<Target = [Field<'a>]>,
    V: ~const Deref<Target = [Variant<'a, F>]>,
    I: ~const Deref<Target = TypeLayoutInfo<'a, F, V>>,
>(
    from: usize,
    value: &TypeLayoutGraph<'a, F, V, I>,
) -> usize {
    // Include the crate version of `type_layout` for cross-version comparison
    let from = serialised_str_len(from, LAYOUT_VERSION);

    let from = serialised_str_len(from, value.ty);

    let mut len = 0;

    while len < value.tys.len() && value.tys[len].is_some() {
        len += 1;
    }

    let mut from = serialised_usize_len(from, len);

    let mut i = 0;

    while i < len {
        if let Some(ty) = &value.tys[i] {
            from = serialised_type_layout_info_len(from, ty);
        }

        i += 1;
    }

    from
}
