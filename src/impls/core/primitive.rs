use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_primitive_type_layout {
    (impl $ty:ty => $val:expr) => {
        unsafe impl TypeLayout for $ty {
            type Inhabited = crate::inhabited::Inhabited;

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                ty: crate::TypeRef::of::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };
        }

        unsafe impl ComputeTypeSet for $ty {
            type Output<T: ExpandTypeSet> = tset![.. @ T];
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

unsafe impl TypeLayout for ! {
    type Inhabited = crate::inhabited::Uninhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl ComputeTypeSet for ! {
    type Output<T: ExpandTypeSet> = tset![.. @ T];
}
