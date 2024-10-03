use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

mod atomic;

unsafe impl<T: TypeLayout> TypeLayout for core::sync::Exclusive<T> {
    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "inner",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::sync::Exclusive<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}
