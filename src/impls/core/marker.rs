use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeHList},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T> TypeLayout for core::marker::PhantomData<T> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
}

unsafe impl<T> ComputeTypeSet for core::marker::PhantomData<T> {
    type Output<R: ExpandTypeHList> = tset![.. @ R];
}

unsafe impl TypeLayout for core::marker::PhantomPinned {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
}

unsafe impl ComputeTypeSet for core::marker::PhantomPinned {
    type Output<T: ExpandTypeHList> = tset![.. @ T];
}
