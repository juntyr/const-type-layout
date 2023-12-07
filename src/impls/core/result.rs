use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: ~const TypeLayout, E: ~const TypeLayout> const TypeLayout
    for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "Ok",
                    discriminant: crate::struct_variant_discriminant!(
                        Result => Result<T, E> => Ok(f_0: T)
                    ),
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Result => Result<T, E> => Ok(f_0: T) => 0),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Err",
                    discriminant: crate::struct_variant_discriminant!(
                        Result => Result<T, E> => Err(f_0: E)
                    ),
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Result => Result<T, E> => Err(f_0: E) => 0),
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        if let MaybeUninhabited::Inhabited(uninit) = <T as TypeLayout>::uninit() {
            return MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(Ok(
                uninit.assume_init()
            )));
        }

        if let MaybeUninhabited::Inhabited(uninit) = <E as TypeLayout>::uninit() {
            return MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(Err(
                uninit.assume_init()
            )));
        }

        MaybeUninhabited::Uninhabited
    }
}

unsafe impl<T: ComputeTypeSet, E: ComputeTypeSet> ComputeTypeSet for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    type Output<R: ExpandTypeSet> = Set<Self, tset![E, .. @ R]>;
}
