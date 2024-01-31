use crate::{
    typeset::{tset, ComputeTypeSet, ExpandTypeSet},
    TypeLayout, TypeLayoutInfo, TypeStructure,
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
        unsafe impl<$R: TypeLayout, $($T: TypeLayout),*> TypeLayout for $ty {
            type Inhabited = crate::inhabited::Inhabited;

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                ty: crate::TypeRef::of::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };
        }

        unsafe impl<$R: ComputeTypeSet, $($T: ComputeTypeSet),*> ComputeTypeSet for $ty {
            type Output<Z: ExpandTypeSet> = tset![$R $(, $T)*, .. @ Z];
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
        unsafe impl<$R: TypeLayout, $($T: TypeLayout),*> TypeLayout
            for unsafe extern $abi fn($($T),*, ...) -> $R
        {
            type Inhabited = crate::inhabited::Inhabited;

            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                ty: crate::TypeRef::of::<Self>(),
                size: ::core::mem::size_of::<Self>(),
                alignment: ::core::mem::align_of::<Self>(),
                structure: TypeStructure::Primitive,
            };
        }

        unsafe impl<$R: ComputeTypeSet, $($T: ComputeTypeSet),*> ComputeTypeSet
            for unsafe extern $abi fn($($T),*, ...) -> $R
        {
            type Output<Z: ExpandTypeSet> = tset![$R $(, $T)*, .. @ Z];
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
