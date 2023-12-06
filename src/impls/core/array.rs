use crate::{
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout, const N: usize> const TypeLayout for [T; N] {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        let mut uninit_array: [core::mem::MaybeUninit<T>; N] =
            core::mem::MaybeUninit::uninit_array();

        let mut i = 0;

        while i < N {
            uninit_array[i] = match <T as TypeLayout>::uninit() {
                MaybeUninhabited::Uninhabited => {
                    core::mem::forget(uninit_array);

                    return MaybeUninhabited::Uninhabited;
                },
                MaybeUninhabited::Inhabited(uninit) => uninit,
            };

            i += 1;
        }

        MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(
            core::mem::MaybeUninit::array_assume_init(uninit_array),
        ))
    }
}

unsafe impl<T: ~const TypeGraph, const N: usize> const TypeGraph for [T; N] {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: ComputeTypeSet, const N: usize> ComputeTypeSet for [T; N] {
    type Output<R: ComputeSet> = Set<Self, tset![T, .. @ R]>;
}
