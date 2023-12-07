use crate::{
    typeset::{ComputeTypeSet, ExpandTypeSet, Set},
    MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

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

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        MaybeUninhabited::Uninhabited
    }
}

unsafe impl ComputeTypeSet for core::ffi::c_void {
    type Output<T: ExpandTypeSet> = Set<Self, T>;
}
