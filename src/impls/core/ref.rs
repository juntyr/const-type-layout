use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a T {
    type Static = &'static T::Static;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: false,
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new({
        alloc::boxed::Box::leak(core::mem::ManuallyDrop::into_inner(
            <alloc::boxed::Box<T> as TypeLayout>::UNINIT,
        ))
    });
}

unsafe impl<'a, T: ~const TypeGraph + 'a> const TypeGraph for &'a T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<'a, T: TypeLayout + 'a> TypeLayout for &'a mut T {
    type Static = &'static mut T::Static;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
    // FIXME: constructing invalid value at .value: encountered mutable reference in
    // a `const`
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new({
        alloc::boxed::Box::leak(core::mem::ManuallyDrop::into_inner(
            <alloc::boxed::Box<T> as TypeLayout>::UNINIT,
        ))
    });
}

unsafe impl<'a, T: ~const TypeGraph + 'a> const TypeGraph for &'a mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
