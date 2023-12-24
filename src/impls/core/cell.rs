use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout> TypeLayout for core::cell::UnsafeCell<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "no_nieche,transparent",
            fields: &[Field {
                name: "value",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::UnsafeCell<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::cell::Cell<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<core::cell::UnsafeCell<T>>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::Cell<T> {
    type Output<R: ExpandTypeSet> = tset![core::cell::UnsafeCell<T>, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::cell::SyncUnsafeCell<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "value",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<core::cell::UnsafeCell<T>>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::SyncUnsafeCell<T> {
    type Output<R: ExpandTypeSet> = tset![core::cell::UnsafeCell<T>, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::cell::OnceCell<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[Field {
                name: "inner",
                offset: MaybeUninhabited::new::<T>(0),
                ty: ::core::any::type_name::<core::cell::UnsafeCell<core::option::Option<T>>>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cell::OnceCell<T> {
    type Output<R: ExpandTypeSet> = tset![core::cell::UnsafeCell<core::option::Option<T>>, .. @ R];
}
