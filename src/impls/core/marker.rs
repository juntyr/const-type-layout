use crate::{
    typeset::{ComputeTypeSet, ExpandTypeSet, Set},
    MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T> const TypeLayout for core::marker::PhantomData<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(core::marker::PhantomData::<T>))
    }
}

unsafe impl<T> ComputeTypeSet for core::marker::PhantomData<T> {
    type Output<R: ExpandTypeSet> = Set<Self, R>;
}

unsafe impl const TypeLayout for core::marker::PhantomPinned {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(core::marker::PhantomPinned))
    }
}

unsafe impl ComputeTypeSet for core::marker::PhantomPinned {
    type Output<T: ExpandTypeSet> = Set<Self, T>;
}
