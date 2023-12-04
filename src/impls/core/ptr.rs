use crate::{
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    Field, MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
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

unsafe impl<T: ~const TypeGraph> const TypeGraph for *const T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *const T {
    type Output<R: ComputeSet> = Set<Self, tset!([T] => R)>;
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

unsafe impl<T: ~const TypeGraph> const TypeGraph for *mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *mut T {
    type Output<R: ComputeSet> = Set<Self, tset!([T] => R)>;
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

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::ptr::NonNull<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <*const T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::ptr::NonNull<T> {
    type Output<R: ComputeSet> = Set<Self, tset!([*const T] => R)>;
}
