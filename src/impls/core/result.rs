use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout, E: TypeLayout> TypeLayout for core::result::Result<T, E> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::any![T, E];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "Ok",
                    discriminant: MaybeUninhabited::new::<T>(crate::Discriminant::new::<Self>(0)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<T>(::core::mem::offset_of!(Self, Ok.0)),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Err",
                    discriminant: MaybeUninhabited::new::<E>(crate::Discriminant::new::<Self>(1)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<E>(::core::mem::offset_of!(Self, Err.0)),
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };
}

unsafe impl<T: ComputeTypeSet, E: ComputeTypeSet> ComputeTypeSet for core::result::Result<T, E> {
    type Output<R: ExpandTypeSet> = tset![
        T, E, <Self as crate::ExtractDiscriminant>::Discriminant, .. @ R
    ];
}
