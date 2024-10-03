use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_nonzero_type_layout {
    (impl $nz:ident => $ty:ty) => {
        unsafe impl TypeLayout for core::num::$nz {
            const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];

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
        }

        unsafe impl ComputeTypeSet for core::num::$nz {
            type Output<T: ExpandTypeSet> = tset![$ty, .. @ T];
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

unsafe impl<T: TypeLayout> TypeLayout for core::num::Wrapping<T> {
    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::num::Wrapping<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::num::Saturating<T> {
    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::num::Saturating<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}
