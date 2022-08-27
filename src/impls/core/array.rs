use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout, const N: usize> TypeLayout for [T; N] {
    type Static = [T::Static; N];

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Array {
            item: ::core::any::type_name::<T>(),
            len: N,
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new([Self::ELEM; N]);
}

unsafe impl<T: ~const TypeGraph, const N: usize> const TypeGraph for [T; N] {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

trait ArrayElem<T: TypeLayout> {
    const ELEM: T;
}

impl<T: TypeLayout, const N: usize> ArrayElem<T> for [T; N] {
    const ELEM: T = core::mem::ManuallyDrop::into_inner(<T as TypeLayout>::UNINIT);
}
