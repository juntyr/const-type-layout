use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

trait OptionDiscriminant: Sized
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    const NONE_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
    const SOME_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
}

impl<T: TypeLayout> OptionDiscriminant for Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    const NONE_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = {
        let uninit: Self = None;

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
            core::mem::transmute(core::mem::discriminant(&uninit))
        };

        core::mem::forget(uninit);

        let mut big_endian_bytes = [0_u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];

        let mut i = 0;

        while i < system_endian_bytes.len() {
            big_endian_bytes[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                i
            } else {
                system_endian_bytes.len() - i - 1
            }];

            i += 1;
        }

        big_endian_bytes
    };
    const SOME_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = {
        let uninit = Some(core::mem::ManuallyDrop::into_inner(
            <T as TypeLayout>::UNINIT,
        ));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
            core::mem::transmute(core::mem::discriminant(&uninit))
        };

        core::mem::forget(uninit);

        let mut big_endian_bytes = [0_u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];

        let mut i = 0;

        while i < system_endian_bytes.len() {
            big_endian_bytes[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                i
            } else {
                system_endian_bytes.len() - i - 1
            }];

            i += 1;
        }

        big_endian_bytes
    };
}

unsafe impl<T: TypeLayout> TypeLayout for core::option::Option<T>
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
                    name: "None",
                    discriminant: Discriminant {
                        big_endian_bytes: &<Self as OptionDiscriminant>::NONE_DISCRIMINANT_BYTES,
                    },
                    fields: &[],
                },
                Variant {
                    name: "Some",
                    discriminant: Discriminant {
                        big_endian_bytes: &<Self as OptionDiscriminant>::SOME_DISCRIMINANT_BYTES,
                    },
                    fields: &[Field {
                        name: "0",
                        offset: {
                            let uninit = Some(core::mem::ManuallyDrop::into_inner(
                                <T as TypeLayout>::UNINIT,
                            ));
                            let base_ptr: *const Self = core::ptr::addr_of!(uninit).cast();

                            let field_ptr: *const u8 = match &uninit {
                                Some(val) => (val as *const T).cast(),
                                _ => unreachable!(),
                            };

                            #[allow(clippy::cast_sign_loss)]
                            let offset = unsafe {
                                field_ptr.cast::<u8>().offset_from(base_ptr.cast()) as usize
                            };

                            core::mem::forget(uninit);

                            offset
                        },
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
            ],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(None);
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::option::Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
