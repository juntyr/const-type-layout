use crate::{
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    Field, MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_nonzero_type_layout {
    (impl $nz:ident => $ty:ty) => {
        unsafe impl const TypeLayout for core::num::$nz {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: "transparent",
                    fields: &[
                        Field {
                            name: "0",
                            offset: MaybeUninhabited::Inhabited(0),
                            ty: ::core::any::type_name::<$ty>(),
                        },
                    ],
                },
            };

            unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
                MaybeUninhabited::Inhabited(
                    core::mem::MaybeUninit::new(Self::new(1).unwrap())
                )
            }
        }

        unsafe impl const TypeGraph for core::num::$nz {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    <$ty as TypeGraph>::populate_graph(graph);
                }
            }
        }

        unsafe impl ComputeTypeSet for core::num::$nz {
            type Output<T: ComputeSet> = Set<Self, tset!([$ty] => T)>;
        }
    };
    ($($nz:ident => $ty:ty),*) => {
        $(impl_nonzero_type_layout!{impl $nz => $ty})*
    };
}

impl_nonzero_type_layout! {
    NonZeroI8 => i8, NonZeroI16 => i16, NonZeroI32 => i32, NonZeroI64 => i64,
    NonZeroI128 => i128, NonZeroIsize => isize,
    NonZeroU8 => u8, NonZeroU16 => u16, NonZeroU32 => u32, NonZeroU64 => u64,
    NonZeroU128 => u128, NonZeroUsize => usize
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::num::Wrapping<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: unsafe { <T as TypeLayout>::uninit() }.map(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        match <T as TypeLayout>::uninit() {
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
            MaybeUninhabited::Inhabited(uninit) => {
                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(Self(uninit.assume_init())))
            },
        }
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::num::Wrapping<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::num::Wrapping<T> {
    type Output<R: ComputeSet> = Set<Self, tset!([T] => R)>;
}
