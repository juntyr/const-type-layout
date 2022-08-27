use crate::{Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout> TypeLayout for core::pin::Pin<T> {
    type Static = core::pin::Pin<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "pointer",
                offset: 0,
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(Self {
        pointer: core::mem::ManuallyDrop::into_inner(T::UNINIT),
    });
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::pin::Pin<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
