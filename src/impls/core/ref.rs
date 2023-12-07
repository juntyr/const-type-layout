use crate::{
    impls::leak_uninit_ptr,
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<'a, T: ~const TypeLayout + 'a> const TypeLayout for &'a T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        if let MaybeUninhabited::Uninhabited = <T as TypeLayout>::uninit() {
            return MaybeUninhabited::Uninhabited;
        }

        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(&*leak_uninit_ptr()))
    }
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<'a, T: ~const TypeLayout + 'a> const TypeLayout for &'a mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        if let MaybeUninhabited::Uninhabited = <T as TypeLayout>::uninit() {
            return MaybeUninhabited::Uninhabited;
        }

        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(&mut *leak_uninit_ptr()))
    }
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a mut T {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}
