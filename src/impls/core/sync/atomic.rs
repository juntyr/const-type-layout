use crate::{Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

macro_rules! impl_atomic_int_layout {
    (impl $at:ident ( $align:literal : $cfg:literal ) => $ty:ty) => {
        #[cfg(target_has_atomic_load_store = $cfg)]
        unsafe impl TypeLayout for core::sync::atomic::$at {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: concat!("align(", $align, "),C"),
                    fields: &[
                        Field {
                            name: "v",
                            offset: 0,
                            ty: core::any::type_name::<core::cell::UnsafeCell<$ty>>(),
                        },
                    ],
                },
            };
        }

        #[cfg(target_has_atomic_load_store = $cfg)]
        unsafe impl const TypeGraph for core::sync::atomic::$at {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    <core::cell::UnsafeCell<$ty> as TypeGraph>::populate_graph(graph);
                }
            }
        }
    };
    ($($at:ident ( $align:literal : $cfg:literal ) => $ty:ty),*) => {
        $(impl_atomic_int_layout!{impl $at ($align : $cfg) => $ty})*
    };
}

impl_atomic_int_layout! {
    AtomicBool (1:"8") => u8,
    AtomicI8 (1:"8") => i8, AtomicI16 (2:"16") => i16,
    AtomicI32 (4:"32") => i32, AtomicI64 (8:"64") => i64,
    AtomicU8 (1:"8") => u8, AtomicU16 (2:"16") => u16,
    AtomicU32 (4:"32") => u32, AtomicU64 (8:"64") => u64,
    AtomicI128 (16:"128") => i128, AtomicU128 (16:"128") => u128
}

macro_rules! impl_atomic_ptr_layout {
    (impl $at:ident ( $align:literal : $cfg:literal ) => $ty:ty) => {
        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl TypeLayout for core::sync::atomic::$at {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: concat!("align(", $align, "),C"),
                    fields: &[
                        Field {
                            name: "v",
                            offset: 0,
                            ty: core::any::type_name::<core::cell::UnsafeCell<$ty>>(),
                        },
                    ],
                },
            };
        }

        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl const TypeGraph for core::sync::atomic::$at {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    <core::cell::UnsafeCell<$ty> as TypeGraph>::populate_graph(graph);
                }
            }
        }
    };
    ($($at:ident ( $align:literal : $cfg:literal ) => $ty:ty),*) => {
        $(impl_atomic_ptr_layout!{impl $at ($align : $cfg) => $ty})*
    };
}

impl_atomic_ptr_layout! {
    AtomicIsize (2:"16") => isize, AtomicIsize (4:"32") => isize,
    AtomicIsize (8:"64") => isize,
    AtomicUsize (2:"16") => usize, AtomicUsize (4:"32") => usize,
    AtomicUsize (8:"64") => usize
}
