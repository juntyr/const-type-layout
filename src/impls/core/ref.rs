use crate::{graph::hlist, TypeLayout, TypeLayoutInfo, TypeStructure};

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a T {
    type TypeGraphEdges = hlist![T];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a mut T {
    type TypeGraphEdges = hlist![T];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}
