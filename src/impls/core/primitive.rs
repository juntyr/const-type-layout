use crate::{graph::hlist, TypeLayout, TypeLayoutInfo, TypeStructure};

macro_rules! impl_primitive_type_layout {
    (impl $ty:ty => $val:expr) => {
        unsafe impl TypeLayout for $ty {
            const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };

            type TypeGraphEdges = hlist![];
        }
    };
    ($($ty:ty => $val:expr),*) => {
        $(impl_primitive_type_layout!{impl $ty => $val})*
    };
}

impl_primitive_type_layout! {
    i8 => 0, i16 => 0, i32 => 0, i64 => 0, i128 => 0, isize => 0,
    u8 => 0, u16 => 0, u32 => 0, u64 => 0, u128 => 0, usize => 0,
    f32 => 0.0, f64 => 0.0,
    char => '\0', bool => false, () => ()
}

#[cfg(feature = "impl-never")]
unsafe impl TypeLayout for ! {
    type TypeGraphEdges = hlist![];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::any![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}
