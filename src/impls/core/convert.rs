use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl TypeLayout for core::convert::Infallible {
    type Static = Self;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
    #[allow(unreachable_code, clippy::empty_loop)]
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(loop {});
}

unsafe impl const TypeGraph for core::convert::Infallible {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}
