use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

trait ResultDiscriminant: Sized
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    const OK_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
    const ERR_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
}

impl<T, E> ResultDiscriminant for Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    const ERR_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
        let mut value: core::mem::MaybeUninit<E> = core::mem::MaybeUninit::uninit();

        let mut i = 0;
        while i < core::mem::size_of::<E>() {
            *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
            i += 1;
        }

        let err: core::mem::MaybeUninit<Self> =
            core::mem::MaybeUninit::new(Err(value.assume_init()));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            core::mem::transmute(core::mem::discriminant(err.assume_init_ref()));

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
    const OK_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
        let mut value: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();

        let mut i = 0;
        while i < core::mem::size_of::<T>() {
            *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
            i += 1;
        }

        let ok: core::mem::MaybeUninit<Self> = core::mem::MaybeUninit::new(Ok(value.assume_init()));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            core::mem::transmute(core::mem::discriminant(ok.assume_init_ref()));

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

unsafe impl<T, E> TypeLayout for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
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
                        big_endian_bytes: &<Self as ResultDiscriminant>::OK_DISCRIMINANT_BYTES,
                    },
                    fields: &[Field {
                        name: "0",
                        offset: unsafe {
                            let mut value: core::mem::MaybeUninit<T> =
                                core::mem::MaybeUninit::uninit();

                            let mut i = 0;
                            while i < core::mem::size_of::<T>() {
                                *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
                                i += 1;
                            }

                            let ok: core::mem::MaybeUninit<Self> =
                                core::mem::MaybeUninit::new(Ok(value.assume_init()));

                            #[allow(clippy::cast_sign_loss)]
                            match ok.assume_init_ref() {
                                Ok(val) => (val as *const T)
                                    .cast::<u8>()
                                    .offset_from(ok.as_ptr().cast())
                                    as usize,
                                _ => unreachable!(),
                            }
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
                        offset: unsafe {
                            let mut value: core::mem::MaybeUninit<E> =
                                core::mem::MaybeUninit::uninit();

                            let mut i = 0;
                            while i < core::mem::size_of::<E>() {
                                *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
                                i += 1;
                            }

                            let err: core::mem::MaybeUninit<Self> =
                                core::mem::MaybeUninit::new(Err(value.assume_init()));

                            #[allow(clippy::cast_sign_loss)]
                            match err.assume_init_ref() {
                                Err(val) => (val as *const E)
                                    .cast::<u8>()
                                    .offset_from(err.as_ptr().cast())
                                    as usize,
                                _ => unreachable!(),
                            }
                        },
                        ty: ::core::any::type_name::<E>(),
                    }],
                },
            ],
        },
    };
}

unsafe impl<T: ~const TypeGraph, E: ~const TypeGraph> const TypeGraph for core::result::Result<T, E>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
            <E as TypeGraph>::populate_graph(graph);
        }
    }
}
