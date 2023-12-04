//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//! [![License Status]][fossa] [![Code Coverage]][codecov]
//! [![Gitpod Ready-to-Code]][gitpod]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/const-type-layout/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/const-type-layout/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.60.0-orange
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
//! # #![feature(cfg_version)]
//! # #![feature(const_type_name)]
//! # #![feature(const_refs_to_cell)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_mut_refs)]
//! # #![cfg_attr(not(version("1.61.0")), feature(const_fn_trait_bound))]
//! # #![cfg_attr(not(version("1.61.0")), feature(const_ptr_offset))]
//! # #![allow(incomplete_features)]
//! # #![feature(generic_const_exprs)]
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
//! # #![feature(cfg_version)]
//! # #![feature(const_type_name)]
//! # #![feature(const_refs_to_cell)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_mut_refs)]
//! # #![cfg_attr(not(version("1.61.0")), feature(const_fn_trait_bound))]
//! # #![cfg_attr(not(version("1.61.0")), feature(const_ptr_offset))]
//! # #![allow(incomplete_features)]
//! # #![feature(generic_const_exprs)]
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
//!     name: "rust_out::main::_doctest_main_src_lib_rs_98_0::OverAligned",
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
#![feature(cfg_version)]
#![feature(const_type_name)]
#![cfg_attr(not(version("1.61.0")), feature(const_ptr_offset))]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![cfg_attr(not(version("1.61.0")), feature(const_fn_trait_bound))]
#![feature(cfg_target_has_atomic)]
#![feature(const_discriminant)]
#![cfg_attr(not(version("1.65.0")), feature(const_ptr_offset_from))]
#![feature(const_refs_to_cell)]
#![feature(const_option)]
#![cfg_attr(not(version("1.66.0")), feature(let_else))]
#![feature(core_intrinsics)]
#![feature(const_heap)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(allocator_api)]
#![feature(const_pin)]
#![feature(const_ptr_write)]
#![feature(inline_const)]
#![feature(const_eval_select)]
#![feature(never_type)]
#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_maybe_uninit_array_assume_init)]
#![feature(c_variadic)]
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

use core::{marker::Destruct, ops::Deref};

use alloc::fmt;

pub use const_type_layout_derive::TypeLayout;

#[doc(hidden)]
pub mod impls;
mod ser;
#[cfg(feature = "serde")]
mod serde;
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

impl<T> MaybeUninhabited<T> {
    #[must_use]
    pub const fn map<U: ~const Destruct>(&self, value: U) -> MaybeUninhabited<U> {
        match self {
            Self::Inhabited(_) => MaybeUninhabited::Inhabited(value),
            Self::Uninhabited => MaybeUninhabited::Uninhabited,
        }
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
pub unsafe trait TypeLayout: Sized + typeset::ComputeTypeSet {
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

/// # Safety
///
/// It is only safe to implement this trait if it accurately populates the
///  type's layout graph. Use `#[derive(TypeLayout)]` instead.
#[const_trait]
pub unsafe trait TypeGraph: ~const TypeLayout {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>);
}

#[const_trait]
pub trait TypeGraphLayout: ~const TypeLayout + ~const TypeGraph {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: ~const TypeLayout + ~const TypeGraph> const TypeGraphLayout for T {
    const TYPE_GRAPH: TypeLayoutGraph<'static> = {
        let mut graph = TypeLayoutGraph::new::<T>();

        <T as TypeGraph>::populate_graph(&mut graph);

        graph
    };
}

#[must_use]
pub const fn serialised_type_graph_len<T: ~const TypeGraphLayout>() -> usize {
    T::TYPE_GRAPH.serialised_len()
}

#[must_use]
pub const fn serialise_type_graph<T: ~const TypeGraphLayout>(
) -> [u8; serialised_type_graph_len::<T>()] {
    let mut bytes = [0_u8; serialised_type_graph_len::<T>()];

    T::TYPE_GRAPH.serialise(&mut bytes);

    bytes
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "F: ::serde::Serialize, V: ::serde::Serialize, I: ::serde::Serialize"
    ))
)]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "'a: 'de, F: ::serde::Deserialize<'a>, V: \
                               ::serde::Deserialize<'a>, I: ::serde::Deserialize<'a>"))
)]
pub struct TypeLayoutGraph<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    V: Deref<Target = [Variant<'a, F>]> = &'a [Variant<'a, F>],
    I: Deref<Target = TypeLayoutInfo<'a, F, V>> = &'a TypeLayoutInfo<'a, F, V>,
> {
    ty: &'a str,
    #[cfg_attr(feature = "serde", serde(with = "serde"))]
    tys: [Option<I>; TypeLayoutGraph::CAPACITY],
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Variant<'a, F: Deref<Target = [Field<'a>]> = &'a [Field<'a>]> {
    pub name: &'a str,
    pub discriminant: MaybeUninhabited<Discriminant<'a>>,
    pub fields: F,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(transparent)]
pub struct Discriminant<'a> {
    pub big_endian_bytes: &'a [u8],
}

#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Field<'a> {
    pub name: &'a str,
    pub offset: MaybeUninhabited<usize>,
    pub ty: &'a str,
}

impl<'a> TypeLayoutGraph<'a> {
    const CAPACITY: usize = 1024;

    #[must_use]
    #[doc(hidden)]
    pub const fn new<T: ~const TypeLayout>() -> Self {
        Self {
            ty: <T as TypeLayout>::TYPE_LAYOUT.name,
            tys: [None; Self::CAPACITY],
        }
    }

    #[doc(hidden)]
    pub const fn insert(&mut self, ty: &'a TypeLayoutInfo<'a>) -> bool {
        let ty_name_bytes = ty.name.as_bytes();

        let mut i = 0;

        while i < self.tys.len() {
            // The first free slot can be used to insert the ty
            let Some(cached_ty) = self.tys[i] else {
                self.tys[i] = Some(ty);

                return true;
            };

            let cached_ty_name_bytes = cached_ty.name.as_bytes();

            // The type names can only be equal if they are the same length
            if ty_name_bytes.len() == cached_ty_name_bytes.len() {
                let mut j = 0;

                while j < ty_name_bytes.len() {
                    // Break early, i.e. j < ty_name_bytes.len(),
                    //  if the type names do NOT match
                    if ty_name_bytes[j] != cached_ty_name_bytes[j] {
                        break;
                    }

                    j += 1;
                }

                // j == ty_name_bytes.len() IFF the type names match
                if j >= ty_name_bytes.len() {
                    return false;
                }
            }

            i += 1;
        }

        panic!("TypeLayoutGraph is not large enough for this complex type.")
    }
}

impl<
        'a,
        F: Deref<Target = [Field<'a>]>,
        V: Deref<Target = [Variant<'a, F>]>,
        I: Deref<Target = TypeLayoutInfo<'a, F, V>>,
    > TypeLayoutGraph<'a, F, V, I>
{
    #[must_use]
    pub const fn serialised_len(&self) -> usize
    where
        F: ~const Deref<Target = [Field<'a>]>,
        V: ~const Deref<Target = [Variant<'a, F>]>,
        I: ~const Deref<Target = TypeLayoutInfo<'a, F, V>>,
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
        I: ~const Deref<Target = TypeLayoutInfo<'a, F, V>>,
    {
        let from = ser::serialise_usize(bytes, 0, self.serialised_len());

        ser::serialise_type_layout_graph(bytes, from, self);
    }
}

impl<
        'a,
        F: Deref<Target = [Field<'a>]> + fmt::Debug,
        V: Deref<Target = [Variant<'a, F>]> + fmt::Debug,
        I: Deref<Target = TypeLayoutInfo<'a, F, V>> + fmt::Debug,
    > fmt::Debug for TypeLayoutGraph<'a, F, V, I>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "TypeLayoutGraph<{}>(", self.ty)?;

        let mut debug = fmt.debug_list();

        for ty in &self.tys {
            match ty {
                Some(ty) => debug.entry(&ty),
                None => break,
            };
        }

        debug.finish()?;

        write!(fmt, ")")
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

impl<'a> fmt::Debug for Discriminant<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "0x")?;

        let mut is_zero = true;

        for byte in self.big_endian_bytes.iter().copied() {
            if byte != 0_u8 {
                is_zero = false;

                write!(fmt, "{byte:x}")?;
            }
        }

        if is_zero {
            write!(fmt, "0")?;
        }

        Ok(())
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
#[allow_internal_unstable(const_ptr_offset_from)]
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
#[allow_internal_unstable(const_discriminant)]
pub macro struct_variant_discriminant {
    ($ty_name:ident => $ty:ty => $variant_name:ident) => {
        $crate::MaybeUninhabited::Inhabited {
            0: $crate::Discriminant {
                big_endian_bytes: &{
                    let uninit: $ty = $ty_name::$variant_name;

                    let system_endian_bytes: [u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()] = unsafe {
                        ::core::mem::transmute(::core::mem::discriminant(&uninit))
                    };

                    #[allow(clippy::forget_non_drop, clippy::forget_copy)]
                    ::core::mem::forget(uninit);

                    let mut big_endian_bytes = [0_u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()];

                    let mut i = 0;

                    while i < system_endian_bytes.len() {
                        big_endian_bytes[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                            i
                        } else {
                            system_endian_bytes.len() - i - 1
                        }];

                        i += 1;
                    }

                    big_endian_bytes
                },
            },
        }
    },
    ($ty_name:ident => $ty:ty => $variant_name:ident($($field_name:ident: $field_ty:ty),* $(,)?)) => {{
        #[allow(unused_parens)]
        if let (
            $($crate::MaybeUninhabited::Inhabited($field_name)),*
        ) = (
            $(unsafe { <$field_ty as $crate::TypeLayout>::uninit() }),*
        ) {
            $crate::MaybeUninhabited::Inhabited {
                0: $crate::Discriminant {
                    big_endian_bytes: {
                        let uninit: $ty = $ty_name::$variant_name(
                            $(unsafe { $field_name.assume_init() }),*
                        );

                        let system_endian_bytes: [u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()] = unsafe {
                            ::core::mem::transmute(::core::mem::discriminant(&uninit))
                        };

                        #[allow(clippy::forget_non_drop, clippy::forget_copy)]
                        ::core::mem::forget(uninit);

                        let big_endian_bytes = unsafe {
                            &mut *$crate::impls::leak_uninit_ptr::<
                                [u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()]
                            >()
                        };

                        let mut i = 0;

                        while i < system_endian_bytes.len() {
                            (*big_endian_bytes)[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                                i
                            } else {
                                system_endian_bytes.len() - i - 1
                            }];

                            i += 1;
                        }

                        big_endian_bytes
                    }
                },
            }
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
            $crate::MaybeUninhabited::Inhabited {
                0: $crate::Discriminant {
                    big_endian_bytes: {
                        let uninit: $ty = $ty_name::$variant_name {
                            $($field_name: unsafe { $field_name.assume_init() }),*
                        };

                        let system_endian_bytes: [u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()] = unsafe {
                            ::core::mem::transmute(::core::mem::discriminant(&uninit))
                        };

                        #[allow(clippy::forget_non_drop, clippy::forget_copy)]
                        ::core::mem::forget(uninit);

                        let big_endian_bytes = unsafe {
                            &mut *$crate::impls::leak_uninit_ptr::<
                                [u8; ::core::mem::size_of::<::core::mem::Discriminant<$ty>>()]
                            >()
                        };

                        let mut i = 0;

                        while i < system_endian_bytes.len() {
                            (*big_endian_bytes)[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                                i
                            } else {
                                system_endian_bytes.len() - i - 1
                            }];

                            i += 1;
                        }

                        big_endian_bytes
                    }
                },
            }
        } else {
            $crate::MaybeUninhabited::Uninhabited
        }
    }},
}

#[doc(hidden)]
#[allow_internal_unstable(const_ptr_offset_from)]
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
