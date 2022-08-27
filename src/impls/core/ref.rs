use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: false,
        },
    };
    #[allow(clippy::borrow_as_ptr)]
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(unsafe {
        &*(&<T as TypeLayout>::UNINIT as *const core::mem::ManuallyDrop<T>).cast::<T>()
    });
}

unsafe impl<'a, T: ~const TypeGraph + 'static> const TypeGraph for &'a T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
    #[allow(const_item_mutation, clippy::borrow_as_ptr)]
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(unsafe {
        &mut *(&mut <T as TypeLayout>::UNINIT as *mut core::mem::ManuallyDrop<T>).cast::<T>()
    });
}

unsafe impl<'a, T: ~const TypeGraph + 'static> const TypeGraph for &'a mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
