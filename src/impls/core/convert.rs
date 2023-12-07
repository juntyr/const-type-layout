use crate::{
    typeset::{ComputeTypeSet, ExpandTypeSet, Set},
    MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl const TypeLayout for core::convert::Infallible {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Uninhabited
    }
}

unsafe impl ComputeTypeSet for core::convert::Infallible {
    type Output<T: ExpandTypeSet> = Set<Self, T>;
}
