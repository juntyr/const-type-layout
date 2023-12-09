//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//! [![License Status]][fossa] [![Code Coverage]][codecov]
//! [![Gitpod Ready-to-Code]][gitpod]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/const-type-layout/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/const-type-layout/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.75.0--nightly-orange
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
//! # #![feature(offset_of)]
//! # #![feature(offset_of_enum)]
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
//! # #![feature(offset_of)]
//! # #![feature(offset_of_enum)]
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
//!     name: "rust_out::main::_doctest_main_src_lib_rs_92_0::OverAligned",
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
#![feature(cfg_target_has_atomic)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(c_variadic)]
#![feature(ptr_from_ref)]
#![feature(discriminant_kind)]
#![feature(offset_of)]
#![feature(offset_of_enum)]
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

mod impls;
pub mod inhabited;
mod ser;
pub mod typeset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub enum MaybeUninhabited<T> {
    Uninhabited,
    Inhabited(T),
}

impl<T: Copy> MaybeUninhabited<T> {
    #[must_use]
    pub const fn new<U: TypeLayout>(v: T) -> Self {
        if <U::Inhabited as Same<inhabited::Inhabited>>::EQ {
            Self::Inhabited(v)
        } else {
            Self::Uninhabited
        }
    }
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
pub unsafe trait TypeLayout: Sized {
    type Inhabited: inhabited::OutputMaybeInhabited;

    const TYPE_LAYOUT: TypeLayoutInfo<'static>;
}

pub trait TypeGraphLayout: TypeLayout + typeset::ComputeTypeSet {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: TypeLayout + typeset::ComputeTypeSet> TypeGraphLayout for T {
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
    pub ty: &'a str,
    pub tys: G,
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
#[cfg_attr(feature = "serde", allow(clippy::unsafe_derive_deserialize))]
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

impl Discriminant {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub const fn new<T: ExtractDiscriminant>(v: T::Ty) -> Self {
        union Transmute<T: Copy> {
            v: T,
            i8: i8,
            i16: i16,
            i32: i32,
            i64: i64,
            i128: i128,
            isize: isize,
            u8: u8,
            u16: u16,
            u32: u32,
            u64: u64,
            u128: u128,
            usize: usize,
        }

        if <T::Ty as Same<i8>>::EQ {
            return Self::I8(unsafe { Transmute { v }.i8 });
        } else if <T::Ty as Same<i16>>::EQ {
            return Self::I16(unsafe { Transmute { v }.i16 });
        } else if <T::Ty as Same<i32>>::EQ {
            return Self::I32(unsafe { Transmute { v }.i32 });
        } else if <T::Ty as Same<i64>>::EQ {
            return Self::I64(unsafe { Transmute { v }.i64 });
        } else if <T::Ty as Same<i128>>::EQ {
            return Self::I128(unsafe { Transmute { v }.i128 });
        } else if <T::Ty as Same<isize>>::EQ {
            return Self::Isize(unsafe { Transmute { v }.isize });
        } else if <T::Ty as Same<u8>>::EQ {
            return Self::U8(unsafe { Transmute { v }.u8 });
        } else if <T::Ty as Same<u16>>::EQ {
            return Self::U16(unsafe { Transmute { v }.u16 });
        } else if <T::Ty as Same<u32>>::EQ {
            return Self::U32(unsafe { Transmute { v }.u32 });
        } else if <T::Ty as Same<u64>>::EQ {
            return Self::U64(unsafe { Transmute { v }.u64 });
        } else if <T::Ty as Same<u128>>::EQ {
            return Self::U128(unsafe { Transmute { v }.u128 });
        } else if <T::Ty as Same<usize>>::EQ {
            return Self::Usize(unsafe { Transmute { v }.usize });
        }

        panic!("bug: unknown discriminant kind")
    }
}

trait Same<T> {
    const EQ: bool;
}

impl<T, U> Same<U> for T {
    default const EQ: bool = false;
}

impl<T> Same<T> for T {
    const EQ: bool = true;
}

pub trait ExtractDiscriminant {
    type Ty: Copy + typeset::ComputeTypeSet;
}

impl<T> ExtractDiscriminant for T {
    type Ty =
        <T as ExtractDiscriminantSpec<<T as core::marker::DiscriminantKind>::Discriminant>>::Ty;
}

#[doc(hidden)]
pub trait ExtractDiscriminantSpec<T> {
    type Ty: Copy + typeset::ComputeTypeSet;
}

impl<T> ExtractDiscriminantSpec<<T as core::marker::DiscriminantKind>::Discriminant> for T {
    default type Ty = !;
}

macro_rules! impl_extract_discriminant {
    ($variant:ident($ty:ty)) => {
        impl<T: core::marker::DiscriminantKind<Discriminant = $ty>> ExtractDiscriminantSpec<$ty> for T {
            type Ty = $ty;
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
    pub const fn new<T: TypeLayout + typeset::ComputeTypeSet>() -> Self {
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
        // F: Deref<Target = [Field<'a>]>,
        // V: Deref<Target = [Variant<'a, F>]>,
        // P: Deref<Target = [&'a str]>,
        // I: Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
        // G: Deref<Target = [I]>,
    >
    TypeLayoutGraph<
        'a,
        // F, V, P, I, G,
    >
{
    #[must_use]
    pub const fn serialised_len(&self) -> usize
// where
    //     F: ~const Deref<Target = [Field<'a>]>,
    //     V: ~const Deref<Target = [Variant<'a, F>]>,
    //     P: ~const Deref<Target = [&'a str]>,
    //     I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
    //     G: ~const Deref<Target = [I]>,
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
    // where
    //     F: ~const Deref<Target = [Field<'a>]>,
    //     V: ~const Deref<Target = [Variant<'a, F>]>,
    //     P: ~const Deref<Target = [&'a str]>,
    //     I: ~const Deref<Target = TypeLayoutInfo<'a, F, V, P>>,
    //     G: ~const Deref<Target = [I]>,
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
