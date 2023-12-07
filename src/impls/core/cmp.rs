use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::cmp::Reverse<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: unsafe { <T as TypeLayout>::uninit() }.map(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        match <T as TypeLayout>::uninit() {
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
            MaybeUninhabited::Inhabited(uninit) => {
                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(Self(uninit.assume_init())))
            },
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cmp::Reverse<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}
