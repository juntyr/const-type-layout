use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: ~const TypeLayout, const N: usize> const TypeLayout for [T; N] {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Array {
            item: ::core::any::type_name::<T>(),
            len: N,
        },
    };

    unsafe fn uninit() -> core::mem::ManuallyDrop<Self> {
        core::mem::ManuallyDrop::new(
            [const { core::mem::ManuallyDrop::into_inner(unsafe { <T as TypeLayout>::uninit() }) };
                N],
        )
    }
}

unsafe impl<T: ~const TypeGraph, const N: usize> const TypeGraph for [T; N] {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
