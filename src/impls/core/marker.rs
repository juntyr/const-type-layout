use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T> TypeLayout for core::marker::PhantomData<T> {
    type Inhabited = crate::inhabited::Inhabited;

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
    type Output<R: ExpandTypeSet> = tset![.. @ R];
}

unsafe impl TypeLayout for core::marker::PhantomPinned {
    type Inhabited = crate::inhabited::Inhabited;

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
    type Output<T: ExpandTypeSet> = tset![.. @ T];
}
