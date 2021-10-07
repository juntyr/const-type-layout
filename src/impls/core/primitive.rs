use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

macro_rules! impl_primitive_type_layout {
    (impl $ty:ty) => {
        unsafe impl TypeLayout for $ty {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };
        }

        unsafe impl const TypeGraph for $ty {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                graph.insert(&Self::TYPE_LAYOUT);
            }
        }
    };
    ($($ty:ty),*) => {
        $(impl_primitive_type_layout!{impl $ty})*
    };
}

impl_primitive_type_layout! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    char, bool, ()
}
