use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl TypeLayout for core::convert::Infallible {
    type Inhabited = crate::inhabited::Uninhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
}

unsafe impl ComputeTypeSet for core::convert::Infallible {
    type Output<T: ExpandTypeSet> = tset![.. @ T];
}
