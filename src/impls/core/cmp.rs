use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<T: TypeLayout> TypeLayout for core::cmp::Reverse<T> {
    type Inhabited = T::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "transparent",
            fields: &[Field {
                name: "0",
                offset: MaybeUninhabited::new::<T>(0),
                ty: crate::TypeRef::of::<T>(),
            }],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::cmp::Reverse<T> {
    type Output<R: ExpandTypeSet> = tset![T, .. @ R];
}

unsafe impl TypeLayout for core::cmp::Ordering {
    type Inhabited = crate::inhabited::Inhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "i8",
            variants: &[
                Variant {
                    name: "Less",
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(-1)),
                    fields: &[],
                },
                Variant {
                    name: "Equal",
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(0)),
                    fields: &[],
                },
                Variant {
                    name: "Greater",
                    discriminant: MaybeUninhabited::Inhabited(crate::Discriminant::new::<Self>(1)),
                    fields: &[],
                },
            ],
        },
    };
}

unsafe impl ComputeTypeSet for core::cmp::Ordering {
    type Output<R: ExpandTypeSet> = tset![
        <Self as crate::ExtractDiscriminant>::Discriminant, .. @ R
    ];
}
