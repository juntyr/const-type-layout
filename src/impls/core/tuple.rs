use crate::{
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    Field, MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_tuple_type_layout {
    (impl ($($a:tt => $T:ident),+)) => {
        unsafe impl<$($T: ~const TypeLayout),*> const TypeLayout for ($($T,)*) {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                // Even though tuples are primitives, their field layout is non-trivial
                structure: TypeStructure::Struct {
                    repr: "",
                    fields: &[$(Field {
                        name: stringify!($a),
                        offset: match unsafe { <Self as crate::TypeLayout>::uninit() } {
                            MaybeUninhabited::Uninhabited => MaybeUninhabited::Uninhabited,
                            MaybeUninhabited::Inhabited(uninit) => {
                                let base: *const Self = core::ptr::addr_of!(uninit).cast();
                                let field_ptr = unsafe { core::ptr::addr_of!((*base).$a) };

                                #[allow(clippy::cast_sign_loss)]
                                let offset = unsafe { field_ptr.cast::<u8>().offset_from(base.cast()) as usize };

                                #[allow(clippy::forget_non_drop, clippy::forget_copy)]
                                core::mem::forget(uninit);

                                MaybeUninhabited::Inhabited(offset)
                            },
                        },
                        ty: ::core::any::type_name::<$T>(),
                    }),*],
                },
            };

            unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
                $(
                    #[allow(non_snake_case)]
                    let MaybeUninhabited::Inhabited($T) = <$T as crate::TypeLayout>::uninit() else {
                        return MaybeUninhabited::Uninhabited;
                    };
                )*

                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(
                    ($($T.assume_init(),)*)
                ))
            }
        }

        unsafe impl<$($T: ~const TypeGraph),*> const TypeGraph for ($($T,)*) {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    $(<$T as TypeGraph>::populate_graph(graph);)*
                }
            }
        }

        unsafe impl<$($T: ComputeTypeSet),*> ComputeTypeSet for ($($T,)*) {
            type Output<T: ComputeSet> = Set<Self, tset!([$($T),*] => T)>;
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
