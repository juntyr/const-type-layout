use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for *const T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(
            core::ptr::NonNull::dangling().as_ptr(),
        ))
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *const T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for *mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(
            core::ptr::NonNull::dangling().as_ptr(),
        ))
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *mut T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::ptr::NonNull<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "pointer",
                offset: MaybeUninhabited::Inhabited(0),
                ty: ::core::any::type_name::<*const T>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(core::ptr::NonNull::dangling()))
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::ptr::NonNull<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![*const T, .. @ R]>;
}
