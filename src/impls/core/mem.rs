use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout> TypeLayout for core::mem::ManuallyDrop<T> {
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
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::mem::ManuallyDrop<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![T, .. @ R]>;
}

unsafe impl<T: TypeLayout> TypeLayout for core::mem::MaybeUninit<T> {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Union {
            repr: "transparent",
            fields: &[
                Field {
                    name: "uninit",
                    offset: MaybeUninhabited::Inhabited(0),
                    ty: ::core::any::type_name::<()>(),
                },
                Field {
                    name: "value",
                    offset: MaybeUninhabited::new::<T>(0),
                    ty: ::core::any::type_name::<core::mem::ManuallyDrop<T>>(),
                },
            ],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::mem::MaybeUninit<T> {
    type Output<R: ExpandTypeSet> = Set<Self, tset![(), core::mem::ManuallyDrop<T>, .. @ R]>;
}
