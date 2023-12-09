use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout + core::ops::Deref> const TypeLayout for core::pin::Pin<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "pointer",
                offset: match unsafe { <T as TypeLayout>::uninit() } {
                    MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(0),
                    MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
                },
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        match <T as TypeLayout>::uninit() {
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
            MaybeUninhabited::Inhabited(uninit) => MaybeUninhabited::Inhabited(
                core::mem::MaybeUninit::new(Self::new_unchecked(uninit.assume_init())),
            ),
        }
    }
}

unsafe impl<T: ComputeTypeSet + core::ops::Deref> ComputeTypeSet for core::pin::Pin<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}
