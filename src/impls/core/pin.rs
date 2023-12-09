use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout + core::ops::Deref> TypeLayout for core::pin::Pin<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "pointer",
                // TODO: check for uninhabited
                offset: MaybeUninhabited::Inhabited(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet + core::ops::Deref> ComputeTypeSet for core::pin::Pin<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}
