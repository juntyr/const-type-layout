use crate::{
    typeset::{ComputeTypeSet, ExpandTypeSet, Set},
    MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_primitive_type_layout {
    (impl $ty:ty => $val:expr) => {
        unsafe impl const TypeLayout for $ty {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };

            unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new($val))
            }
        }

        unsafe impl const TypeGraph for $ty {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                graph.insert(&Self::TYPE_LAYOUT);
            }
        }

        unsafe impl ComputeTypeSet for $ty {
            type Output<T: ExpandTypeSet> = Set<Self, T>;
        }
    };
    ($($ty:ty => $val:expr),*) => {
        $(impl_primitive_type_layout!{impl $ty => $val})*
    };
}

impl_primitive_type_layout! {
    i8 => 0, i16 => 0, i32 => 0, i64 => 0, i128 => 0, isize => 0,
    u8 => 0, u16 => 0, u32 => 0, u64 => 0, u128 => 0, usize => 0,
    f32 => 0.0, f64 => 0.0,
    char => '\0', bool => false, () => ()
}

unsafe impl const TypeLayout for ! {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Uninhabited
    }
}

unsafe impl const TypeGraph for ! {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}

unsafe impl ComputeTypeSet for ! {
    type Output<T: ExpandTypeSet> = Set<Self, T>;
}
