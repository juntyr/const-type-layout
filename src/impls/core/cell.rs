use crate::{graph::hlist, Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout> TypeLayout for core::cell::UnsafeCell<T> {
    type TypeGraphEdges = hlist![T];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
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

unsafe impl<T: TypeLayout> TypeLayout for core::cell::Cell<T> {
    type TypeGraphEdges = hlist![core::cell::UnsafeCell<T>];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
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

#[cfg(feature = "impl-sync-unsafe-cell")]
unsafe impl<T: TypeLayout> TypeLayout for core::cell::SyncUnsafeCell<T> {
    type TypeGraphEdges = hlist![core::cell::UnsafeCell<T>];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
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

unsafe impl<T: TypeLayout> TypeLayout for core::cell::OnceCell<T> {
    type TypeGraphEdges = hlist![core::cell::UnsafeCell<core::option::Option<T>>];

    const INHABITED: crate::MaybeUninhabited = T::INHABITED;
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
