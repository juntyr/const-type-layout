use crate::{
    impls::leak_uninit_ptr, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for alloc::boxed::Box<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(alloc::boxed::Box::from_raw_in(
            leak_uninit_ptr(),
            alloc::alloc::Global,
        ))
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for alloc::boxed::Box<[T]> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };

    #[allow(clippy::borrow_as_ptr)]
    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(alloc::boxed::Box::from_raw_in(
            &[] as *const [T] as *mut _,
            alloc::alloc::Global,
        ))
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<[T]> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
