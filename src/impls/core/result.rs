use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

// TODO: needs specialisation for uninhabited case?
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
                    discriminant: Discriminant {
                        big_endian_bytes: &crate::struct_variant_discriminant!(
                            Result => Result<T, E> => Ok(T)
                        ),
                    },
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Result => Result<T, E> => Ok(T) => 0),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Err",
                    discriminant: Discriminant {
                        big_endian_bytes: &crate::struct_variant_discriminant!(
                            Result => Result<T, E> => Err(E)
                        ),
                    },
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Result => Result<T, E> => Err(E) => 0),
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };

    unsafe fn uninit() -> core::mem::ManuallyDrop<Self> {
        core::mem::ManuallyDrop::new(Ok(core::mem::ManuallyDrop::into_inner(
            <T as TypeLayout>::uninit(),
        )))
    }
}

unsafe impl<T: ~const TypeGraph + ~const TypeLayout, E: ~const TypeGraph + ~const TypeLayout> const
    TypeGraph for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
            <E as TypeGraph>::populate_graph(graph);
        }
    }
}
