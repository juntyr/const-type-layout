use crate::{graph::hlist, TypeLayout, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout, const N: usize> TypeLayout for [T; N] {
    type TypeGraphEdges = hlist![T];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}
