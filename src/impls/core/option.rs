use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

// TODO: needs specialisation for uninhabited case?
unsafe impl<T: ~const TypeLayout> const TypeLayout for core::option::Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    type Static = core::option::Option<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "None",
                    discriminant: Discriminant {
                        big_endian_bytes: &crate::struct_variant_discriminant!(
                            Option => Option<T> => None
                        ),
                    },
                    fields: &[],
                },
                Variant {
                    name: "Some",
                    discriminant: Discriminant {
                        big_endian_bytes: &crate::struct_variant_discriminant!(
                            Option => Option<T> => Some(T)
                        ),
                    },
                    fields: &[Field {
                        name: "0",
                        offset: crate::struct_variant_field_offset!(Option => Option<T> => Some(T) => 0),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
            ],
        },
    };

    unsafe fn uninit() -> core::mem::ManuallyDrop<Self> {
        core::mem::ManuallyDrop::new(None)
    }
}

unsafe impl<T: ~const TypeGraph + ~const TypeLayout> const TypeGraph for core::option::Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
