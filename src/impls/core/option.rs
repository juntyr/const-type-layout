use crate::{
    Discriminant, Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
    Variant,
};

trait OptionDiscriminant: Sized
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    const NONE_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
    const SOME_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()];
}

impl<T> OptionDiscriminant for Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    const NONE_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
        let none: core::mem::MaybeUninit<Self> = core::mem::MaybeUninit::new(None);

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            core::mem::transmute(core::mem::discriminant(none.assume_init_ref()));

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

    const SOME_DISCRIMINANT_BYTES: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] = unsafe {
        let mut value: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();

        let mut i = 0;
        while i < core::mem::size_of::<T>() {
            *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
            i += 1;
        }

        let some: core::mem::MaybeUninit<Self> =
            core::mem::MaybeUninit::new(Some(value.assume_init()));

        let system_endian_bytes: [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()] =
            core::mem::transmute(core::mem::discriminant(some.assume_init_ref()));

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

unsafe impl<T> TypeLayout for core::option::Option<T>
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
                        offset: unsafe {
                            let mut value: core::mem::MaybeUninit<T> =
                                core::mem::MaybeUninit::uninit();

                            let mut i = 0;
                            while i < core::mem::size_of::<T>() {
                                *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
                                i += 1;
                            }

                            let some: core::mem::MaybeUninit<Self> =
                                core::mem::MaybeUninit::new(Some(value.assume_init()));

                            #[allow(clippy::cast_sign_loss)]
                            match some.assume_init_ref() {
                                Some(val) => (val as *const T)
                                    .cast::<u8>()
                                    .offset_from(some.as_ptr().cast())
                                    as usize,
                                _ => unreachable!(),
                            }
                        },
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
            ],
        },
    };
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::option::Option<T>
where
    [u8; core::mem::size_of::<core::mem::Discriminant<Self>>()]: ,
{
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
