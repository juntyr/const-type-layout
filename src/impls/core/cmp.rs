use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout> TypeLayout for core::cmp::Reverse<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cmp::Reverse<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}
