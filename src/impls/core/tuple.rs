use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet, Set},
    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_tuple_type_layout {
    (impl ($($a:tt => $T:ident),+)) => {
        unsafe impl<$($T: TypeLayout),*> TypeLayout for ($($T,)*) {
            type Inhabited = crate::inhabited::all![$($T),*];

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                // Even though tuples are primitives, their field layout is non-trivial
                structure: TypeStructure::Struct {
                    repr: "",
                    fields: &[$(Field {
                        name: stringify!($a),
                        offset: MaybeUninhabited::new::<$T>(core::mem::offset_of!(Self, $a)),
                        ty: ::core::any::type_name::<$T>(),
                    }),*],
                },
            };
        }

        unsafe impl<$($T: ComputeTypeSet),*> ComputeTypeSet for ($($T,)*) {
            type Output<T: ExpandTypeSet> = Set<Self, tset![$($T),*, .. @ T]>;
        }
    };
    ($(($($a:tt => $T:ident),+)),*) => {
        $(impl_tuple_type_layout!{impl ($($a => $T),*)})*
    };
}

impl_tuple_type_layout! {
    (0=>A),
    (0=>A, 1=>B),
    (0=>A, 1=>B, 2=>C),
    (0=>A, 1=>B, 2=>C, 3=>D),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G, 7=>H),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G, 7=>H, 8=>I),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G, 7=>H, 8=>I, 9=>J),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G, 7=>H, 8=>I, 9=>J, 10=>K),
    (0=>A, 1=>B, 2=>C, 3=>D, 4=>E, 5=>F, 6=>G, 7=>H, 8=>I, 9=>J, 10=>K, 11=>L)
}
