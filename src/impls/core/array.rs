use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: ~const TypeLayout, const N: usize> const TypeLayout for [T; N] {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        inhabited: T::TYPE_LAYOUT.inhabited,
        structure: TypeStructure::Array {
            item: ::core::any::type_name::<T>(),
            len: N,
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::new(
            [const { unsafe { <T as TypeLayout>::uninit().assume_init() } }; N],
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
