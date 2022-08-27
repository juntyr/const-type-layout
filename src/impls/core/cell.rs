use crate::{Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout> TypeLayout for core::cell::UnsafeCell<T> {
    type Static = core::cell::UnsafeCell<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "no_nieche,transparent",
            fields: &[Field {
                name: "value",
                offset: 0,
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> =
        core::mem::ManuallyDrop::new(Self::new(core::mem::ManuallyDrop::into_inner(T::UNINIT)));
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::cell::UnsafeCell<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: TypeLayout> TypeLayout for core::cell::Cell<T> {
    type Static = core::cell::Cell<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: 0,
                ty: ::core::any::type_name::<core::cell::UnsafeCell<T>>(),
            }],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> =
        core::mem::ManuallyDrop::new(Self::new(core::mem::ManuallyDrop::into_inner(T::UNINIT)));
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::cell::Cell<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <core::cell::UnsafeCell<T> as TypeGraph>::populate_graph(graph);
        }
    }
}
