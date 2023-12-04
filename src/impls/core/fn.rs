use crate::{
    typeset::{tset, ComputeSet, ComputeTypeSet, Set},
    MaybeUninhabited, TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure,
};

macro_rules! impl_fn_pointer_type_layout {
    (impl extern $abi:literal fn($($T:ident),*) -> $R:ident) => {
        impl_fn_pointer_type_layout!{
            impl extern $abi fn($($T),*) -> $R,
            extern $abi fn($($T),*) -> $R,
            extern $abi fn demo<$R, $($T),*>($(_: $T),*) -> $R { loop {} }
        }
    };
    (impl unsafe extern $abi:literal fn($($T:ident),*) -> $R:ident) => {
        impl_fn_pointer_type_layout!{
            impl extern $abi fn($($T),*) -> $R,
            unsafe extern $abi fn($($T),*) -> $R,
            unsafe extern $abi fn demo<$R, $($T),*>($(_: $T),*) -> $R { loop {} }
        }
    };
    (impl extern $abi:literal fn($($T:ident),*) -> $R:ident, $ty:ty, $demo:item) => {
        unsafe impl<$R: ~const TypeLayout, $($T: ~const TypeLayout),*> const TypeLayout for $ty {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };

            unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
                #[allow(clippy::too_many_arguments)]
                $demo

                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(demo))
            }
        }

        unsafe impl<$R: ~const TypeGraph, $($T: ~const TypeGraph),*> const TypeGraph for $ty {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    <$R as TypeGraph>::populate_graph(graph);
                    $(<$T as TypeGraph>::populate_graph(graph);)*
                }
            }
        }

        unsafe impl<$R: ComputeTypeSet, $($T: ComputeTypeSet),*> ComputeTypeSet for $ty {
            type Output<Z: ComputeSet> = Set<Self, tset!([$R $(, $T)*] => Z)>;
        }
    };
    ($(fn($($T:ident),*) -> $R:ident),*) => {
        $(impl_fn_pointer_type_layout!{impl extern "Rust" fn($($T),*) -> $R})*
        $(impl_fn_pointer_type_layout!{impl unsafe extern "Rust" fn($($T),*) -> $R})*
        $(impl_fn_pointer_type_layout!{impl extern "C" fn($($T),*) -> $R})*
        $(impl_fn_pointer_type_layout!{impl unsafe extern "C" fn($($T),*) -> $R})*
    };
}

impl_fn_pointer_type_layout! {
    fn() -> R,
    fn(A) -> R,
    fn(A, B) -> R,
    fn(A, B, C) -> R,
    fn(A, B, C, D) -> R,
    fn(A, B, C, D, E) -> R,
    fn(A, B, C, D, E, F) -> R,
    fn(A, B, C, D, E, F, G) -> R,
    fn(A, B, C, D, E, F, G, H) -> R,
    fn(A, B, C, D, E, F, G, H, I) -> R,
    fn(A, B, C, D, E, F, G, H, I, J) -> R,
    fn(A, B, C, D, E, F, G, H, I, J, K) -> R,
    fn(A, B, C, D, E, F, G, H, I, J, K, L) -> R
}

macro_rules! impl_variadic_extern_fn_pointer_type_layout {
    (impl unsafe extern $abi:literal fn($($T:ident),+, ...) -> $R:ident) => {
        unsafe impl<$R: ~const TypeLayout, $($T: ~const TypeLayout),*> const TypeLayout
            for unsafe extern $abi fn($($T),*, ...) -> $R
        {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };

            unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>> {
                #[allow(clippy::too_many_arguments)]
                unsafe extern $abi fn demo<$R, $($T),*>(
                    $(_: $T),*, _: ...
                ) -> $R { loop {} }

                MaybeUninhabited::Inhabited(core::mem::MaybeUninit::new(demo))
            }
        }

        unsafe impl<$R: ~const TypeGraph, $($T: ~const TypeGraph),*> const TypeGraph
            for unsafe extern $abi fn($($T),*, ...) -> $R
        {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    <$R as TypeGraph>::populate_graph(graph);
                    $(<$T as TypeGraph>::populate_graph(graph);)*
                }
            }
        }

        unsafe impl<$R: ComputeTypeSet, $($T: ComputeTypeSet),*> ComputeTypeSet
            for unsafe extern $abi fn($($T),*, ...) -> $R
        {
            type Output<Z: ComputeSet> = Set<Self, tset!([$R $(, $T)*] => Z)>;
        }
    };
    ($(unsafe extern "C" fn($($T:ident),+, ...) -> $R:ident),*) => {
        $(impl_variadic_extern_fn_pointer_type_layout!{
            impl unsafe extern "C" fn($($T),*, ...) -> $R
        })*
    };
}

impl_variadic_extern_fn_pointer_type_layout! {
    unsafe extern "C" fn(A, ...) -> R,
    unsafe extern "C" fn(A, B, ...) -> R,
    unsafe extern "C" fn(A, B, C, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, H, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, H, I, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, H, I, J, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, H, I, J, K, ...) -> R,
    unsafe extern "C" fn(A, B, C, D, E, F, G, H, I, J, K, L, ...) -> R
}
