use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl const TypeLayout for core::ffi::c_void {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "u8",
            variants: &[],
        },
    };

    unsafe fn uninit() -> core::mem::MaybeUninit<Self> {
        panic!("cannot construct core::ffi::c_void")
    }
}

unsafe impl const TypeGraph for core::ffi::c_void {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}
