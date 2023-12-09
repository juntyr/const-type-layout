use crate::{
    typeset::{ComputeTypeSet, ExpandTypeSet, Set},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl TypeLayout for core::convert::Infallible {
    type Inhabited = crate::inhabited::Uninhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
}

unsafe impl ComputeTypeSet for core::convert::Infallible {
    type Output<T: ExpandTypeSet> = Set<Self, T>;
}
