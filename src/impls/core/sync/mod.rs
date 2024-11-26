#[cfg(feature = "impl-sync-exclusive")]
use crate::{graph::hlist, Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure};

#[cfg(feature = "impl-atomics")]
mod atomic;

#[cfg(feature = "impl-sync-exclusive")]
unsafe impl<T: TypeLayout> TypeLayout for core::sync::Exclusive<T> {
    type TypeGraphEdges = hlist![T];

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
