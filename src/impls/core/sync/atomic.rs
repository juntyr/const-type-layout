use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_atomic_int_layout {
    (impl $at:ident ( $align:literal : $cfg:literal ) => $ty:ty => $val:literal) => {
        #[cfg(target_has_atomic_load_store = $cfg)]
        unsafe impl TypeLayout for core::sync::atomic::$at {
            const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: concat!("C,align(", $align, ")"),
                    fields: &[
                        Field {
                            name: "v",
                            offset: MaybeUninhabited::Inhabited(0),
                            ty: core::any::type_name::<core::cell::UnsafeCell<$ty>>(),
                        },
                    ],
                },
            };
        }

        #[cfg(target_has_atomic_load_store = $cfg)]
        unsafe impl ComputeTypeSet for core::sync::atomic::$at {
            type Output<T: ExpandTypeSet> = tset![core::cell::UnsafeCell<$ty>, .. @ T];
        }
    };
    ($($at:ident ( $align:literal : $cfg:literal ) => $ty:ty => $val:literal),*) => {
        $(impl_atomic_int_layout!{impl $at ($align : $cfg) => $ty => $val})*
    };
}

impl_atomic_int_layout! {
    AtomicBool (1:"8") => u8 => false,
    AtomicI8 (1:"8") => i8 => 0, AtomicI16 (2:"16") => i16 => 0,
    AtomicI32 (4:"32") => i32 => 0, AtomicI64 (8:"64") => i64 => 0,
    AtomicU8 (1:"8") => u8 => 0, AtomicU16 (2:"16") => u16 => 0,
    AtomicU32 (4:"32") => u32 => 0, AtomicU64 (8:"64") => u64 => 0
}

#[cfg(feature = "impl-atomics-128")]
impl_atomic_int_layout! {
    AtomicI128 (16:"128") => i128 => 0, AtomicU128 (16:"128") => u128 => 0
}

macro_rules! impl_atomic_int_ptr_sized_layout {
    (impl $at:ident ( $align:literal : $cfg:literal ) => $ty:ty => $val:literal) => {
        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl TypeLayout for core::sync::atomic::$at {
            const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: concat!("C,align(", $align, ")"),
                    fields: &[
                        Field {
                            name: "v",
                            offset: MaybeUninhabited::Inhabited(0),
                            ty: core::any::type_name::<core::cell::UnsafeCell<$ty>>(),
                        },
                    ],
                },
            };
        }

        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl ComputeTypeSet for core::sync::atomic::$at {
            type Output<T: ExpandTypeSet> = tset![core::cell::UnsafeCell<$ty>, .. @ T];
        }
    };
    ($($at:ident ( $align:literal : $cfg:literal ) => $ty:ty => $val:literal),*) => {
        $(impl_atomic_int_ptr_sized_layout!{impl $at ($align : $cfg) => $ty => $val})*
    };
}

impl_atomic_int_ptr_sized_layout! {
    AtomicIsize (2:"16") => isize => 0, AtomicIsize (4:"32") => isize => 0,
    AtomicIsize (8:"64") => isize => 0,
    AtomicUsize (2:"16") => usize => 0, AtomicUsize (4:"32") => usize => 0,
    AtomicUsize (8:"64") => usize => 0
}

macro_rules! impl_atomic_ptr_layout {
    (impl ( $align:literal : $cfg:literal )) => {
        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl<T: TypeLayout> TypeLayout for core::sync::atomic::AtomicPtr<T> {
            const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Struct {
                    repr: concat!("C,align(", $align, ")"),
                    fields: &[
                        Field {
                            name: "v",
                            offset: MaybeUninhabited::Inhabited(0),
                            ty: core::any::type_name::<core::cell::UnsafeCell<*mut T>>(),
                        },
                    ],
                },
            };
        }

        #[cfg(target_has_atomic_load_store = "ptr")]
        #[cfg(target_pointer_width = $cfg)]
        unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::sync::atomic::AtomicPtr<T> {
            type Output<R: ExpandTypeSet> = tset![core::cell::UnsafeCell<T>, .. @ R];
        }
    };
    ($(( $align:literal : $cfg:literal )),*) => {
        $(impl_atomic_ptr_layout!{impl ($align : $cfg)})*
    };
}

impl_atomic_ptr_layout! {
    (2:"16"), (4:"32"), (8:"64")
}
