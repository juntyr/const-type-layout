use crate::{graph::hlist, TypeLayout, TypeLayoutInfo, TypeStructure};

unsafe impl TypeLayout for core::convert::Infallible {
    type TypeGraphEdges = hlist![];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::any![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
}
