//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//! [![License Status]][fossa] [![Code Coverage]][codecov]
//! [![Gitpod Ready-to-Code]][gitpod]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/const-type-layout/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/const-type-layout/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.70.0-orange
//! [repo]: https://github.com/juntyr/const-type-layout
//!
//! [Latest Version]: https://img.shields.io/crates/v/const-type-layout
//! [crates.io]: https://crates.io/crates/const-type-layout
//!
//! [Rust Doc Crate]: https://img.shields.io/docsrs/const-type-layout
//! [docs.rs]: https://docs.rs/const-type-layout/
//!
//! [Rust Doc Main]: https://img.shields.io/badge/docs-main-blue
//! [docs]: https://juntyr.github.io/const-type-layout/const_type_layout
//!
//! [License Status]: https://app.fossa.com/api/projects/custom%2B26490%2Fgithub.com%2Fjuntyr%2Fconst-type-layout.svg?type=shield
//! [fossa]: https://app.fossa.com/projects/custom%2B26490%2Fgithub.com%2Fjuntyr%2Fconst-type-layout?ref=badge_shield
//!
//! [Code Coverage]: https://img.shields.io/codecov/c/github/juntyr/const-type-layout?token=J39WVBIMZX
//! [codecov]: https://codecov.io/gh/juntyr/const-type-layout
//!
//! [Gitpod Ready-to-Code]: https://img.shields.io/badge/Gitpod-ready-blue?logo=gitpod
//! [gitpod]: https://gitpod.io/#https://github.com/juntyr/const-type-layout
//!
//! `const-type-layout` is a type layout comparison aid, providing a
//! `#[derive]`able `TypeLayout` trait that reports:
//! - The type's name, size, and minimum alignment
//! - The type's structure, i.e. struct vs. union vs. enum
//! - Each field's name and offset
//! - Each variant's name and discriminant
//!
//! Through the auto-implemented `TypeGraphLayout` trait, the deep type layout
//! is also reported as a graph.
//!
//! This crate heavily builds on the original runtime [type-layout](https://github.com/LPGhatguy/type-layout) crate by Lucien Greathouse.
//!
//! ## Examples
//!
//! The layout of types is only defined if they're `#[repr(C)]`. This crate
//! works on non-`#[repr(C)]` types, but their layout is unpredictable.
//!
//! ```rust
//! # #![feature(const_type_name)]
//! # #![feature(const_refs_to_cell)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_mut_refs)]
//! use const_type_layout::TypeLayout;
//!
//! #[derive(TypeLayout)]
//! #[repr(C)]
//! struct Foo {
//!     a: u8,
//!     b: u32,
//! }
//!
//! assert_eq!(
//!     format!("{:#?}", Foo::TYPE_LAYOUT),
//! r#"TypeLayoutInfo {
//!     name: "rust_out::main::_doctest_main_src_lib_rs_47_0::Foo",
//!     size: 8,
//!     alignment: 4,
//!     structure: Struct {
//!         repr: "C",
//!         fields: [
//!             Field {
//!                 name: "a",
//!                 offset: Inhabited(
//!                     0,
//!                 ),
//!                 ty: "u8",
//!             },
//!             Field {
//!                 name: "b",
//!                 offset: Inhabited(
//!                     4,
//!                 ),
//!                 ty: "u32",
//!             },
//!         ],
//!     },
//! }"#
//! )
//! ```
//!
//! Over-aligned types have trailing padding, which can be a source of bugs in
//! some FFI scenarios:
//!
//! ```rust
//! # #![feature(const_type_name)]
//! # #![feature(const_refs_to_cell)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_mut_refs)]
//! use const_type_layout::TypeLayout;
//!
//! #[derive(TypeLayout)]
//! #[repr(C, align(128))]
//! struct OverAligned {
//!     value: u8,
//! }
//!
//! assert_eq!(
//!     format!("{:#?}", OverAligned::TYPE_LAYOUT),
//! r#"TypeLayoutInfo {
//!     name: "rust_out::main::_doctest_main_src_lib_rs_93_0::OverAligned",
//!     size: 128,
//!     alignment: 128,
//!     structure: Struct {
//!         repr: "C,align(128)",
//!         fields: [
//!             Field {
//!                 name: "value",
//!                 offset: Inhabited(
//!                     0,
//!                 ),
//!                 ty: "u8",
//!             },
//!         ],
//!     },
//! }"#
//! )
//! ```

#![deny(clippy::pedantic)]
#![no_std]
#![feature(const_type_name)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(cfg_target_has_atomic)]
#![feature(const_discriminant)]
#![feature(const_refs_to_cell)]
#![feature(core_intrinsics)]
#![feature(const_heap)]
#![feature(decl_macro)]
#![feature(const_pin)]
#![feature(const_ptr_write)]
#![feature(const_eval_select)]
#![feature(never_type)]
#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_maybe_uninit_array_assume_init)]
#![feature(c_variadic)]
#![feature(ptr_from_ref)]
#![feature(discriminant_kind)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(specialization)]
#![cfg_attr(
    all(doc, not(docsrs)),
    doc(html_root_url = "https://juntyr.github.io/const-type-layout")
)]
#![cfg_attr(feature = "serde", allow(clippy::type_repetition_in_bounds))]

#[doc(hidden)]
pub extern crate alloc;

use core::ops::Deref;

use alloc::fmt;

pub use const_type_layout_derive::TypeLayout;

#[doc(hidden)]
pub mod impls;
mod ser;
pub mod typeset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub enum MaybeUninhabited<T> {
    Uninhabited,
    Inhabited(T),
}

impl<T: Default> Default for MaybeUninhabited<T> {
    fn default() -> Self {
        Self::Inhabited(T::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub enum Constness {
    NonConst,
    Const,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub enum Asyncness {
    Sync,
    Async,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub enum Safety {
    Safe,
    Unsafe,
}

/// # Safety
///
/// It is only safe to implement this trait if it accurately describes the
///  type's layout. Use `#[derive(TypeLayout)]` instead.
#[const_trait]
pub unsafe trait TypeLayout: Sized {
    const TYPE_LAYOUT: TypeLayoutInfo<'static>;

    #[must_use]
    /// # Safety
    ///
    /// 1. The returned value is not safe to be used in any other way than
    ///    to calculate field offsets and discriminants.
    ///
    /// 2. The value and any value built with it must NOT be dropped.
    ///
    /// 3. Must return `MaybeUninhabited::Uninhabited` iff the type is
    ///    uninhabited.
    unsafe fn uninit() -> MaybeUninhabited<core::mem::MaybeUninit<Self>>;
}

#[const_trait]
pub trait TypeGraphLayout: ~const TypeLayout + typeset::ComputeTypeSet {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: ~const TypeLayout + typeset::ComputeTypeSet> const TypeGraphLayout for T {
    const TYPE_GRAPH: TypeLayoutGraph<'static> = TypeLayoutGraph::new::<T>();
}

#[must_use]
pub const fn serialised_type_graph_len<T: TypeGraphLayout>() -> usize {
    T::TYPE_GRAPH.serialised_len()
}

#[must_use]
pub const fn serialise_type_graph<T: TypeGraphLayout>() -> [u8; serialised_type_graph_len::<T>()] {
    let mut bytes = [0_u8; serialised_type_graph_len::<T>()];

    T::TYPE_GRAPH.serialise(&mut bytes);

    bytes
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", allow(clippy::unsafe_derive_deserialize))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(serialize = "F: ::serde::Serialize, V: ::serde::Serialize, P: \
                             ::serde::Serialize, I: ::serde::Serialize, G: ::serde::Serialize"))
)]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "'a: 'de, F: ::serde::Deserialize<'a>, V: \
                               ::serde::Deserialize<'a>, P: ::serde::Deserialize<'a>, I: \
                               ::serde::Deserialize<'a>, G: ::serde::Deserialize<'a>"))
)]
pub struct TypeLayoutGraph<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    V: Deref<Target = [Variant<'a, F>]> = &'a [Variant<'a, F>],
    P: Deref<Target = [&'a str]> = &'a [&'a str],
    I: Deref<Target = TypeLayoutInfo<'a, F, V, P>> = &'a TypeLayoutInfo<'a, F, V, P>,
    G: Deref<Target = [I]> = &'a [I],
> {
    ty: &'a str,
    tys: G,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct TypeLayoutInfo<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    V: Deref<Target = [Variant<'a, F>]> = &'a [Variant<'a, F>],
    P: Deref<Target = [&'a str]> = &'a [&'a str],
> {
    pub name: &'a str,
    pub size: usize,
    pub alignment: usize,
    pub structure: TypeStructure<'a, F, V, P>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum TypeStructure<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    V: Deref<Target = [Variant<'a, F>]> = &'a [Variant<'a, F>],
    P: Deref<Target = [&'a str]> = &'a [&'a str],
> {
    Primitive,
    Struct {
        repr: &'a str,
        fields: F,
    },
    Union {
        repr: &'a str,
        fields: F,
    },
    Enum {
        repr: &'a str,
        variants: V,
    },
    Function {
        constness: Constness,
        asyncness: Asyncness,
        safety: Safety,
        abi: &'a str,
        parameters: P,
        r#return: &'a str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Variant<'a, F: Deref<Target = [Field<'a>]> = &'a [Field<'a>]> {
    pub name: &'a str,
    pub discriminant: MaybeUninhabited<Discriminant>,
    pub fields: F,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Discriminant {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
}

#[const_trait]
pub trait ExtractDiscriminant {
    type Ty: typeset::ComputeTypeSet;

    fn discriminant(&self) -> Discriminant;
}

impl<T> const ExtractDiscriminant for T {
    type Ty =
        <T as ExtractDiscriminantSpec<<T as core::marker::DiscriminantKind>::Discriminant>>::Ty;

    fn discriminant(&self) -> Discriminant {
        <T as ExtractDiscriminantSpec<<T as core::marker::DiscriminantKind>::Discriminant>>::discriminant(self)
    }
}

#[doc(hidden)]
#[const_trait]
pub trait ExtractDiscriminantSpec<T> {
    type Ty: typeset::ComputeTypeSet;

    fn discriminant(&self) -> Discriminant;
}

impl<T> const ExtractDiscriminantSpec<<T as core::marker::DiscriminantKind>::Discriminant> for T {
    default type Ty = !;

    default fn discriminant(&self) -> Discriminant {
        panic!("bug: unknown discriminant kind")
    }
}

macro_rules! impl_extract_discriminant {
    ($variant:ident($ty:ty)) => {
        impl<T: core::marker::DiscriminantKind<Discriminant = $ty>> const ExtractDiscriminantSpec<$ty> for T {
            type Ty = $ty;

            fn discriminant(&self) -> Discriminant {
                Discriminant::$variant(core::intrinsics::discriminant_value(self))
            }
        }
    };
    ($($variant:ident($ty:ty)),*) => {
        $(impl_extract_discriminant! { $variant($ty) })*
    };
}

impl_extract_discriminant! {
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128), Isize(isize),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128), Usize(usize)
}

#[derive(Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Field<'a> {
    pub name: &'a str,
    pub offset: MaybeUninhabited<usize>,
    pub ty: &'a str,
}

impl TypeLayoutGraph<'static> {
    #[must_use]
    pub const fn new<T: ~const TypeLayout + typeset::ComputeTypeSet>() -> Self {
        Self {
            ty: <T as TypeLayout>::TYPE_LAYOUT.name,
            tys: unsafe {
                core::slice::from_raw_parts(
                    core::ptr::from_ref(<typeset::TypeSet<T> as typeset::ComputeSet>::TYS).cast(),
                    <typeset::TypeSet<T> as typeset::ComputeSet>::LEN,
                )
            },
        }
    }
}

impl<
        'a,
        F: Deref<Target = [Field<'a>]>,
        V: Deref<Target = [Variant<'a, F>]>,
        P: Deref<Target = [&'a str]>,
        I: Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
        G: Deref<Target = [I]>,
    > TypeLayoutGraph<'a, F, V, P, I, G>
{
    #[must_use]
    pub const fn serialised_len(&self) -> usize
    where
        F: ~const Deref<Target = [Field<'a>]>,
        V: ~const Deref<Target = [Variant<'a, F>]>,
        P: ~const Deref<Target = [&'a str]>,
        I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
        G: ~const Deref<Target = [I]>,
    {
        let len = ser::serialised_type_layout_graph_len(0, self);

        let mut last_full_len = len;
        let mut full_len = ser::serialised_usize_len(len, last_full_len);

        while full_len != last_full_len {
            last_full_len = full_len;
            full_len = ser::serialised_usize_len(len, last_full_len);
        }

        full_len
    }

    pub const fn serialise(&self, bytes: &mut [u8])
    where
        F: ~const Deref<Target = [Field<'a>]>,
        V: ~const Deref<Target = [Variant<'a, F>]>,
        P: ~const Deref<Target = [&'a str]>,
        I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
        G: ~const Deref<Target = [I]>,
    {
        let from = ser::serialise_usize(bytes, 0, self.serialised_len());

        ser::serialise_type_layout_graph(bytes, from, self);
    }
}

impl<
        'a,
        F: Deref<Target = [Field<'a>]> + fmt::Debug,
        V: Deref<Target = [Variant<'a, F>]> + fmt::Debug,
        P: Deref<Target = [&'a str]> + fmt::Debug,
        I: Deref<Target = TypeLayoutInfo<'a, F, V, P>> + fmt::Debug,
        G: Deref<Target = [I]> + fmt::Debug,
    > fmt::Debug for TypeLayoutGraph<'a, F, V, P, I, G>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("TypeLayoutGraph<{}>({:?})", self.ty, self.tys))
    }
}

impl<'a, F: Deref<Target = [Field<'a>]> + Ord> Ord for Variant<'a, F> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.discriminant, &self.name, &self.fields).cmp(&(
            &other.discriminant,
            &other.name,
            &other.fields,
        ))
    }
}

impl<'a, F: Deref<Target = [Field<'a>]> + PartialOrd> PartialOrd for Variant<'a, F> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (&self.discriminant, &self.name, &self.fields).partial_cmp(&(
            &other.discriminant,
            &other.name,
            &other.fields,
        ))
    }
}

impl<'a> PartialEq for Field<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.offset == other.offset && core::ptr::eq(self.ty, other.ty)
    }
}

impl<'a> Eq for Field<'a> {}

impl<'a> Ord for Field<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.offset, &self.name, &self.ty).cmp(&(&other.offset, &other.name, &other.ty))
    }
}

impl<'a> PartialOrd for Field<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[doc(hidden)]
pub macro struct_field_offset($ty_name:ident => $ty:ty => (*$base:ident).$field:tt => $($extra_fields:tt)?) {
    {
        #[allow(clippy::unneeded_field_pattern)]
        let $ty_name { $field: _, $($extra_fields)? }: $ty;

        if let $crate::MaybeUninhabited::Inhabited(uninit) = unsafe { <$ty as $crate::TypeLayout>::uninit() } {
            let $base: *const $ty = ::core::ptr::addr_of!(uninit).cast();

            #[allow(unused_unsafe)]
            let field_ptr = unsafe {
                ::core::ptr::addr_of!((*$base).$field)
            };

            #[allow(clippy::cast_sign_loss)]
            let offset = unsafe { field_ptr.cast::<u8>().offset_from($base.cast()) as usize };

            #[allow(clippy::forget_non_drop, clippy::forget_copy)]
            ::core::mem::forget(uninit);

            $crate::MaybeUninhabited::Inhabited(offset)
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }
}

#[doc(hidden)]
pub macro struct_variant_discriminant {
    ($ty_name:ident => $ty:ty => $variant_name:ident) => {{
        let uninit: $ty = $ty_name::$variant_name;

        let discriminant = <$ty as $crate::ExtractDiscriminant>::discriminant(&uninit);

        #[allow(clippy::forget_non_drop, clippy::forget_copy)]
        ::core::mem::forget(uninit);

        $crate::MaybeUninhabited::Inhabited(discriminant)
    }},
    ($ty_name:ident => $ty:ty => $variant_name:ident($($field_name:ident: $field_ty:ty),* $(,)?)) => {{
        #[allow(unused_parens)]
        if let (
            $($crate::MaybeUninhabited::Inhabited($field_name)),*
        ) = (
            $(unsafe { <$field_ty as $crate::TypeLayout>::uninit() }),*
        ) {
            let uninit: $ty = $ty_name::$variant_name(
                $(unsafe { $field_name.assume_init() }),*
            );

            let discriminant = <$ty as $crate::ExtractDiscriminant>::discriminant(&uninit);

            #[allow(clippy::forget_non_drop, clippy::forget_copy)]
            ::core::mem::forget(uninit);

            $crate::MaybeUninhabited::Inhabited(discriminant)
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }},
    ($ty_name:ident => $ty:ty => $variant_name:ident { $($field_name:ident: $field_ty:ty),* $(,)? }) => {{
        #[allow(unused_parens)]
        if let (
            $($crate::MaybeUninhabited::Inhabited($field_name)),*
        ) = (
            $(unsafe { <$field_ty as $crate::TypeLayout>::uninit() }),*
        ) {
            let uninit: $ty = $ty_name::$variant_name {
                $($field_name: unsafe { $field_name.assume_init() }),*
            };

            let discriminant = <$ty as $crate::ExtractDiscriminant>::discriminant(&uninit);

            #[allow(clippy::forget_non_drop, clippy::forget_copy)]
            ::core::mem::forget(uninit);

            $crate::MaybeUninhabited::Inhabited(discriminant)
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }},
}

#[doc(hidden)]
pub macro struct_variant_field_offset {
    ($ty_name:ident => $ty:ty => $variant_name:ident($($field_name:ident: $field_ty:ty),* $(,)?) => $field_index:tt) => {{
        #[allow(unused_parens)]
        if let (
            $($crate::MaybeUninhabited::Inhabited($field_name)),*
        ) = (
            $(unsafe { <$field_ty as $crate::TypeLayout>::uninit() }),*
        ) {
            let uninit: $ty = $ty_name::$variant_name(
                $(unsafe { $field_name.assume_init() }),*
            );
            let base_ptr: *const $ty = ::core::ptr::addr_of!(uninit).cast();

            let field_ptr: *const u8 = match &uninit {
                #[allow(clippy::unneeded_field_pattern, clippy::ptr_as_ptr)]
                $ty_name::$variant_name { $field_index: field, .. } => {
                    field as *const _ as *const u8
                },
                _ => unreachable!(),
            };

            #[allow(clippy::cast_sign_loss)]
            let offset = unsafe { field_ptr.cast::<u8>().offset_from(base_ptr.cast()) as usize };

            #[allow(clippy::forget_non_drop, clippy::forget_copy)]
            ::core::mem::forget(uninit);

            $crate::MaybeUninhabited::Inhabited(offset)
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }},
    ($ty_name:ident => $ty:ty => $variant_name:ident { $($field_name:ident: $field_ty:ty),* $(,)? } => $field_index:ident) => {{
        #[allow(unused_parens)]
        if let (
            $($crate::MaybeUninhabited::Inhabited($field_name)),*
        ) = (
            $(unsafe { <$field_ty as $crate::TypeLayout>::uninit() }),*
        ) {
            let uninit: $ty = $ty_name::$variant_name {
                $($field_name: unsafe { $field_name.assume_init() }),*
            };
            let base_ptr: *const $ty = ::core::ptr::addr_of!(uninit).cast();

            let field_ptr: *const u8 = match &uninit {
                #[allow(clippy::unneeded_field_pattern)]
                $ty_name::$variant_name { $field_index: field, .. } => {
                    field as *const _ as *const u8
                },
                _ => unreachable!(),
            };

            #[allow(clippy::cast_sign_loss)]
            let offset = unsafe { field_ptr.cast::<u8>().offset_from(base_ptr.cast()) as usize };

            #[allow(clippy::forget_non_drop, clippy::forget_copy)]
            ::core::mem::forget(uninit);

            $crate::MaybeUninhabited::Inhabited(offset)
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }},
}
