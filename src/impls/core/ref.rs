use crate::{
    impls::leak_uninit_ptr,
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
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

unsafe impl<'a, T: ~const TypeGraph + 'a> const TypeGraph for &'a T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a T {
    type Output<R: ComputeSet> = Set<Self, tset![T, .. @ R]>;
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

unsafe impl<'a, T: ~const TypeGraph + 'a> const TypeGraph for &'a mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<'a, T: ComputeTypeSet + 'a> ComputeTypeSet for &'a mut T {
    type Output<R: ComputeSet> = Set<Self, tset![T, .. @ R]>;
}
