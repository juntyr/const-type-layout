use crate::{
    graph::hlist, Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout> TypeLayout for core::cmp::Reverse<T> {
    type TypeGraphEdges = hlist![T];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
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

unsafe impl TypeLayout for core::cmp::Ordering {
    type TypeGraphEdges = hlist![::core::mem::Discriminant<Self>];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "i8",
            variants: &[
                Variant {
                    name: "Less",
                    discriminant: MaybeUninhabited::Inhabited(crate::discriminant!(-1)),
                    fields: &[],
                },
                Variant {
                    name: "Equal",
                    discriminant: MaybeUninhabited::Inhabited(crate::discriminant!(0)),
                    fields: &[],
                },
                Variant {
                    name: "Greater",
                    discriminant: MaybeUninhabited::Inhabited(crate::discriminant!(1)),
                    fields: &[],
                },
            ],
        },
    };
}
