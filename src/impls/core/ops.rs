use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure, Variant,
};

unsafe impl<Idx: TypeLayout> TypeLayout for core::ops::Range<Idx> {
    const INHABITED: crate::MaybeUninhabited = Idx::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[
                Field {
                    name: "start",
                    offset: MaybeUninhabited::new::<Idx>(::core::mem::offset_of!(Self, start)),
                    ty: ::core::any::type_name::<Idx>(),
                },
                Field {
                    name: "end",
                    offset: MaybeUninhabited::new::<Idx>(::core::mem::offset_of!(Self, end)),
                    ty: ::core::any::type_name::<Idx>(),
                },
            ],
        },
    };
}

unsafe impl<Idx: ComputeTypeSet> ComputeTypeSet for core::ops::Range<Idx> {
    type Output<R: ExpandTypeSet> = tset![Idx, .. @ R];
}

unsafe impl<Idx: TypeLayout> TypeLayout for core::ops::RangeFrom<Idx> {
    const INHABITED: crate::MaybeUninhabited = Idx::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[Field {
                name: "start",
                offset: MaybeUninhabited::new::<Idx>(::core::mem::offset_of!(Self, start)),
                ty: ::core::any::type_name::<Idx>(),
            }],
        },
    };
}

unsafe impl<Idx: ComputeTypeSet> ComputeTypeSet for core::ops::RangeFrom<Idx> {
    type Output<R: ExpandTypeSet> = tset![Idx, .. @ R];
}

unsafe impl TypeLayout for core::ops::RangeFull {
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

unsafe impl ComputeTypeSet for core::ops::RangeFull {
    type Output<R: ExpandTypeSet> = tset![.. @ R];
}

unsafe impl<Idx: TypeLayout> TypeLayout for core::ops::RangeTo<Idx> {
    const INHABITED: crate::MaybeUninhabited = Idx::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[Field {
                name: "end",
                offset: MaybeUninhabited::new::<Idx>(::core::mem::offset_of!(Self, end)),
                ty: ::core::any::type_name::<Idx>(),
            }],
        },
    };
}

unsafe impl<Idx: ComputeTypeSet> ComputeTypeSet for core::ops::RangeTo<Idx> {
    type Output<R: ExpandTypeSet> = tset![Idx, .. @ R];
}

unsafe impl<Idx: TypeLayout> TypeLayout for core::ops::RangeToInclusive<Idx> {
    const INHABITED: crate::MaybeUninhabited = Idx::INHABITED;
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Struct {
            repr: "",
            fields: &[Field {
                name: "end",
                offset: MaybeUninhabited::new::<Idx>(::core::mem::offset_of!(Self, end)),
                ty: ::core::any::type_name::<Idx>(),
            }],
        },
    };
}

unsafe impl<Idx: ComputeTypeSet> ComputeTypeSet for core::ops::RangeToInclusive<Idx> {
    type Output<R: ExpandTypeSet> = tset![Idx, .. @ R];
}

unsafe impl<T: TypeLayout> TypeLayout for core::ops::Bound<T> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::all![];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "Included",
                    discriminant: MaybeUninhabited::new::<T>(crate::discriminant::discriminant!(0)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<T>(::core::mem::offset_of!(
                            Self, Included.0
                        )),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Excluded",
                    discriminant: MaybeUninhabited::new::<T>(crate::discriminant::discriminant!(1)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<T>(::core::mem::offset_of!(
                            Self, Excluded.0
                        )),
                        ty: ::core::any::type_name::<T>(),
                    }],
                },
                Variant {
                    name: "Unbounded",
                    discriminant: MaybeUninhabited::Inhabited(crate::discriminant::discriminant!(
                        2
                    )),
                    fields: &[],
                },
            ],
        },
    };
}

unsafe impl<T: ComputeTypeSet> ComputeTypeSet for core::ops::Bound<T> {
    type Output<R: ExpandTypeSet> = tset![
        T, ::core::mem::Discriminant<Self>, .. @ R
    ];
}

unsafe impl<B: TypeLayout, C: TypeLayout> TypeLayout for core::ops::ControlFlow<B, C> {
    const INHABITED: crate::MaybeUninhabited = crate::inhabited::any![B, C];
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Enum {
            repr: "",
            variants: &[
                Variant {
                    name: "Continue",
                    discriminant: MaybeUninhabited::new::<C>(crate::discriminant::discriminant!(0)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<C>(::core::mem::offset_of!(
                            Self, Continue.0
                        )),
                        ty: ::core::any::type_name::<C>(),
                    }],
                },
                Variant {
                    name: "Break",
                    discriminant: MaybeUninhabited::new::<B>(crate::discriminant::discriminant!(1)),
                    fields: &[Field {
                        name: "0",
                        offset: MaybeUninhabited::new::<B>(::core::mem::offset_of!(Self, Break.0)),
                        ty: ::core::any::type_name::<B>(),
                    }],
                },
            ],
        },
    };
}

unsafe impl<B: ComputeTypeSet, C: ComputeTypeSet> ComputeTypeSet for core::ops::ControlFlow<B, C> {
    type Output<R: ExpandTypeSet> = tset![
        B, C, ::core::mem::Discriminant<Self>, .. @ R
    ];
}
