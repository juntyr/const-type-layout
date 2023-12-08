use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::option::Option<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "None",
                    discriminant: crate::struct_variant_discriminant!(
                        Option => Option<T> => None
                    ),
                    fields: &[],
                },
                Variant {
                    name: "Some",
                    discriminant: crate::struct_variant_discriminant!(
                        Option => Option<T> => Some(f_0: T)
                    ),
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Option => Option<T> => Some(f_0: T) => 0),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
            ],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(None))
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::option::Option<T> {
    type Output<R: ExpandTypeSet> = Set<
        Self,
        tset![
            T, <Self as crate::ExtractDiscriminant>::Ty, .. @ R
        ],
    >;
}
