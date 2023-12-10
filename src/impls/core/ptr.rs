use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout> TypeLayout for *const T {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *const T {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for *mut T {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for *mut T {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::ptr::NonNull<T> {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "pointer",
                offset: MaybeUninhabited::Inhabited(0),
                ty: ::core::any::type_name::<*const T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::ptr::NonNull<T> {
    type Output<R: ExpandTypeSet> = tset![*const T, .. @ R];
}
