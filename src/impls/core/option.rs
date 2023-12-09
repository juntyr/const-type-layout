use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout> TypeLayout for core::option::Option<T> {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "None",
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(0)),
                    fields: &[],
                },
                Variant {
                    name: "Some",
                    discriminant: MaybeUninhabited::new::<T>(crate::Discriminant::new::<Self>(1)),
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

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::option::Option<T> {
    type Output<R: ExpandTypeSet> = Set<
        Self,
        tset![
            T, <Self as crate::ExtractDiscriminant>::Ty, .. @ R
        ],
    >;
}
