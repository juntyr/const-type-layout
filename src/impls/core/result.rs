use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

trait ResultDiscriminant: Sized
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    const OK_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
    const ERR_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
}

impl<T: TypeLayout, E: TypeLayout> ResultDiscriminant for Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    const ERR_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = {
        let uninit: Self = Err(core::mem::ManuallyDrop::into_inner(
            <E as TypeLayout>::UNINIT,
        ));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            unsafe { core::mem::transmute(core::mem::discriminant(&uninit)) };

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
    const OK_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = {
        let uninit: Self = Ok(core::mem::ManuallyDrop::into_inner(
            <T as TypeLayout>::UNINIT,
        ));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            unsafe { core::mem::transmute(core::mem::discriminant(&uninit)) };

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

// TODO: needs specialisation for uninhabited case?
unsafe impl<T: TypeLayout, E: TypeLayout> TypeLayout for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]:,
{
    type Static = core::result::Result<T::Static, E::Static>;

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
                        big_endian_bytes: &<Self as ResultDiscriminant>::OK_DISCRIMINANT_BYTES,
                    },
                    fields: &[Field {
                        name: "0",
                        offset: {
                            let uninit: Self = Ok(core::mem::ManuallyDrop::into_inner(
                                <T as TypeLayout>::UNINIT,
                            ));
                            let base_ptr: *const Self = core::ptr::addr_of!(uninit).cast();

                            let field_ptr: *const u8 = match &uninit {
                                Ok(val) => (val as *const T).cast(),
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
                Variant {
                    name: "Err",
                    discriminant: Discriminant {
                        big_endian_bytes: &<Self as ResultDiscriminant>::ERR_DISCRIMINANT_BYTES,
                    },
                    fields: &[Field {
                        name: "0",
                        offset: {
                            let uninit: Self = Err(core::mem::ManuallyDrop::into_inner(
                                <E as TypeLayout>::UNINIT,
                            ));
                            let base_ptr: *const Self = core::ptr::addr_of!(uninit).cast();

                            let field_ptr: *const u8 = match &uninit {
                                Err(val) => (val as *const E).cast(),
                                _ => unreachable!(),
                            };

                            #[allow(clippy::cast_sign_loss)]
                            let offset = unsafe {
                                field_ptr.cast::<u8>().offset_from(base_ptr.cast()) as usize
                            };

                            core::mem::forget(uninit);

                            offset
                        },
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(Ok(
        core::mem::ManuallyDrop::into_inner(<T as TypeLayout>::UNINIT),
    ));
}

unsafe impl<T: ~const TypeGraph, E: ~const TypeGraph> const TypeGraph for core::result::Result<T, E>
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
