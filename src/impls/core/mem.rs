use crate::{Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::mem::ManuallyDrop<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: 0,
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(Self::new(T::uninit().assume_init()))
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::mem::ManuallyDrop<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::mem::MaybeUninit<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Union {
            repr: "transparent",
            fields: &[
                Field {
                    name: "uninit",
                    offset: 0,
                    ty: ::core::any::type_name::<()>(),
                },
                Field {
                    name: "value",
                    offset: 0,
                    ty: ::core::any::type_name::<T>(),
                },
            ],
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(T::uninit())
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::mem::MaybeUninit<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <() as TypeGraph>::populate_graph(graph);
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
