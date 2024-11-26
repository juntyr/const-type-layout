use crate::{
    graph::hlist, Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout> TypeLayout for core::option::Option<T> {
    type TypeGraphEdges = hlist![T, ::core::mem::Discriminant<Self>];

    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "None",
                    discriminant: MaybeUninhabited::Inhabited(crate::discriminant!(0)),
                    fields: &[],
                },
                Variant {
                    name: "Some",
                    discriminant: MaybeUninhabited::new::<T>(crate::discriminant!(1)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<T>(::core::mem::offset_of!(Self, Some.0)),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
            ],
        },
    };
}
