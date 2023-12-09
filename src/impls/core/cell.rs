use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::cell::UnsafeCell<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "no_nieche,transparent",
            fields: &[Field {
                name: "value",
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
                core::mem::MaybeUninit::new(Self::new(uninit.assume_init())),
            ),
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::UnsafeCell<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<T: ~const TypeLayout> const TypeLayout for core::cell::Cell<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: match unsafe { <T as TypeLayout>::uninit() } {
                    MaybeUninhabited::Inhabited(_) => MaybeUninhabited::Inhabited(0),
                    MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
                },
                ty: ::core::any::type_name::<core::cell::UnsafeCell<T>>(),
            }],
        },
    };

    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
        match <T as TypeLayout>::uninit() {
            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
            MaybeUninhabited::Inhabited(uninit) => MaybeUninhabited::Inhabited(
                core::mem::MaybeUninit::new(Self::new(uninit.assume_init())),
            ),
        }
    }
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::Cell<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![core::cell::UnsafeCell<T>, .. @ R]>;
}
