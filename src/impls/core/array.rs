use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout, const N: usize> TypeLayout for [T; N] {
    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<T: ComputeTypeSet, const N: usize> ComputeTypeSet for [T; N] {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}
