use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a mut T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}
