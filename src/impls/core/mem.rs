use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

unsafe impl<T: TypeLayout> TypeLayout for core::mem::ManuallyDrop<T> {
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
                ty: ::core::any::type_name::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::mem::ManuallyDrop<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::mem::MaybeUninit<T> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
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
    type Output<R: ExpandTypeSet> = tset![(), core::mem::ManuallyDrop<T>, .. @ R];
}

unsafe impl<T> TypeLayout for core::mem::Discriminant<T> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[Field {
                name: "0",
                offset: MaybeUninhabited::new::<Self>(0),
                ty: ::core::any::type_name::<<Self as crate::ExtractDiscriminant>::Discriminant>(),
            }],
        },
    };
}

unsafe impl<T> ComputeTypeSet for core::mem::Discriminant<T> {
    type Output<R: ExpandTypeSet> =
        tset![<Self as crate::ExtractDiscriminant>::Discriminant, .. @ R];
}
