use crate::{Field, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::cmp::Reverse<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: 0,
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(Self(T::uninit().assume_init()))
    }
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for core::cmp::Reverse<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
