use crate::{
    typeset::{ComputeSet, ComputeTypeSet, Set},
    MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
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

unsafe impl<T> const TypeGraph for core::marker::PhantomData<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}

unsafe impl<T> ComputeTypeSet for core::marker::PhantomData<T> {
    type Output<R: ComputeSet> = Set<Self, R>;
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

unsafe impl const TypeGraph for core::marker::PhantomPinned {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}

unsafe impl ComputeTypeSet for core::marker::PhantomPinned {
    type Output<T: ComputeSet> = Set<Self, T>;
}
