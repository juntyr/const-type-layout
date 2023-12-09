use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout, E: TypeLayout> TypeLayout for core::result::Result<T, E> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "Ok",
                    // TODO: check for uninhabited
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(0)),
                    fields: &[Field {
                        name: "0",
                        // TODO: check for uninhabited
                        offset: MaybeUninhabited::Inhabited(::core::mem::offset_of!(Self, Ok.0)),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Err",
                    // TODO: check for uninhabited
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(1)),
                    fields: &[Field {
                        name: "0",
                        // TODO: check for uninhabited
                        offset: MaybeUninhabited::Inhabited(::core::mem::offset_of!(Self, Err.0)),
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };
}

unsafe impl<T: ComputeTypeSet, E: ComputeTypeSet> ComputeTypeSet for core::result::Result<T, E> {
    type Output<R: ExpandTypeSet> = Set<
        Self,
        tset![
            T, E, <Self as crate::ExtractDiscriminant>::Ty, .. @ R
        ],
    >;
}
