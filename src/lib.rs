//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//! [![License Status]][fossa] [![Code Coverage]][codecov]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/const-type-layout/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/const-type-layout/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.78.0--nightly-orange
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
//! `const-type-layout` is a type layout comparison aid, providing a
//! [`#[derive]`](const_type_layout_derive::TypeLayout)able [`TypeLayout`] trait
//! that provides a const [`TypeLayoutInfo`] struct containing:
//! - The type's name, size, and minimum alignment
//! - The type's structure, i.e. struct vs. union vs. enum
//! - Each field's name and offset
//! - Each variant's name and discriminant
//! - Whether each variant / field is inhabited or uninhabited
//!
//! The auto-implemented [`TypeGraphLayout`] trait also provides a const
//! [`TypeLayoutGraph`] struct that describes the deep type layout, including
//! the layouts of all the types mentioned by this type, e.g. in its fields.
//!
//! This crate heavily builds on the original runtime [type-layout](https://github.com/LPGhatguy/type-layout) crate by Lucien Greathouse.
#![cfg_attr(
    feature = "derive",
    doc = r##"
## Examples

The layout of types is only defined if they're `#[repr(C)]`. This crate works
on non-`#[repr(C)]` types, but their layout is unpredictable.

```rust
# #![feature(const_type_name)]
use const_type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C)]
struct Foo {
    a: u8,
    b: u32,
}

assert_eq!(
    format!("{:#?}", Foo::TYPE_LAYOUT),
r#"TypeLayoutInfo {
    name: "rust_out::main::_doctest_main_src_lib_rs_45_0::Foo",
    size: 8,
    alignment: 4,
    structure: Struct {
        repr: "C",
        fields: [
            Field {
                name: "a",
                offset: Inhabited(
                    0,
                ),
                ty: "u8",
            },
            Field {
                name: "b",
                offset: Inhabited(
                    4,
                ),
                ty: "u32",
            },
        ],
    },
}"#
)
```

Over-aligned types have trailing padding, which can be a source of bugs in some
FFI scenarios:

```rust
# #![feature(const_type_name)]
use const_type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C, align(128))]
struct OverAligned {
    value: u8,
}

assert_eq!(
    format!("{:#?}", OverAligned::TYPE_LAYOUT),
r#"TypeLayoutInfo {
    name: "rust_out::main::_doctest_main_src_lib_rs_88_0::OverAligned",
    size: 128,
    alignment: 128,
    structure: Struct {
        repr: "C,align(128)",
        fields: [
            Field {
                name: "value",
                offset: Inhabited(
                    0,
                ),
                ty: "u8",
            },
        ],
    },
}"#
)
```
"##
)]
#![no_std]
// required core features
#![feature(cfg_version)]
#![feature(const_type_name)]
#![feature(decl_macro)]
#![feature(freeze)]
#![feature(offset_of_enum)]
// required, soon-stabilized features
#![cfg_attr(not(version("1.83")), feature(const_mut_refs))]
#![cfg_attr(not(version("1.82")), feature(offset_of_nested))]
// docs-specific features
#![cfg_attr(doc, feature(doc_auto_cfg))]
// optional feature-gated features
// https://github.com/rust-lang/rust/issues/94039
#![cfg_attr(feature = "impl-atomics", feature(cfg_target_has_atomic))]
#![cfg_attr(
    all(feature = "impl-atomics-128", target_has_atomic = "128"),
    feature(integer_atomics)
)]
#![cfg_attr(feature = "impl-never", feature(never_type))]
#![cfg_attr(feature = "impl-sync-exclusive", feature(exclusive_wrapper))]
#![cfg_attr(feature = "impl-sync-unsafe-cell", feature(sync_unsafe_cell))]
// required INCOMPLETE features
#![allow(incomplete_features)]
#![feature(specialization)]
// optional feature-gated INCOMPLETE features
#![cfg_attr(
    feature = "serialize-to-generic-const-array",
    feature(generic_const_exprs)
)]
// further crate attributes
#![cfg_attr(
    all(doc, not(docsrs)),
    doc(html_root_url = "https://juntyr.github.io/const-type-layout")
)]
#![cfg_attr(feature = "serde", allow(clippy::type_repetition_in_bounds))]

extern crate alloc;

use alloc::fmt;
use core::ops::Deref;

#[cfg(feature = "derive")]
pub use const_type_layout_derive::TypeLayout;

mod discriminant;
mod impls;
pub mod inhabited;
mod ser;
pub mod typeset;

pub use discriminant::Discriminant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
/// Optional value that exists if some other type is
/// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
pub enum MaybeUninhabited<T = ()> {
    /// The type is [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited),
    /// no value.
    Uninhabited,
    /// The type is [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited),
    /// some value of type `T`.
    Inhabited(T),
}

impl<T: Copy> MaybeUninhabited<T> {
    #[must_use]
    /// Construct [`MaybeUninhabited::Inhabited`] iff [`<U as
    /// TypeLayout>::INHABITED`][`TypeLayout::INHABITED`] is
    /// [`MaybeUninhabited::Inhabited`], [`MaybeUninhabited::Uninhabited`]
    /// otherwise.
    pub const fn new<U: TypeLayout>(v: T) -> Self {
        U::INHABITED.map(v)
    }
}

impl MaybeUninhabited {
    #[must_use]
    /// Maps a [`MaybeUninhabited::Inhabited`] to a
    /// [`MaybeUninhabited<T>::Inhabited`] with the value `v` or returns
    /// [`MaybeUninhabited<T>::Uninhabited`].
    pub const fn map<T: Copy>(self, v: T) -> MaybeUninhabited<T> {
        match self {
            Self::Inhabited(()) => MaybeUninhabited::Inhabited(v),
            Self::Uninhabited => MaybeUninhabited::Uninhabited,
        }
    }

    #[must_use]
    /// Returns the logical and between `self` and `other`.
    ///
    /// The result is [`MaybeUninhabited::Inhabited`] iff both `self` and
    /// `other` are [`MaybeUninhabited::Inhabited`],
    /// [`MaybeUninhabited::Uninhabited`] otherwise.
    pub const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Inhabited(()), Self::Inhabited(())) => Self::Inhabited(()),
            _ => Self::Uninhabited,
        }
    }

    #[must_use]
    /// Returns the or and between `self` and `other`.
    ///
    /// The result is [`MaybeUninhabited::Uninhabited`] iff both `self` and
    /// `other` are [`MaybeUninhabited::Uninhabited`],
    /// [`MaybeUninhabited::Inhabited`] otherwise.
    pub const fn or(self, b: Self) -> Self {
        match (self, b) {
            (Self::Uninhabited, Self::Uninhabited) => Self::Uninhabited,
            _ => Self::Inhabited(()),
        }
    }
}

impl<T: Default> Default for MaybeUninhabited<T> {
    fn default() -> Self {
        Self::Inhabited(T::default())
    }
}

/// Utility trait that provides the shallow layout of a type.
///
/// # Safety
///
/// It is only safe to implement this trait if it accurately describes the
///  type's layout. Use
/// [`#[derive(TypeLayout)]`](const_type_layout_derive::TypeLayout) instead.
///
/// # Example
///
/// The struct `Foo` with `u8` and `u16` fields implements [`TypeLayout`] as
/// follows:
///
/// ```rust
/// # #![feature(const_type_name)]
/// # use const_type_layout::{
/// #    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
/// # };
/// # use const_type_layout::inhabited;
/// # use const_type_layout::typeset::{ComputeTypeSet, ExpandTypeSet, tset};
/// struct Foo {
///     a: u8,
///     b: u16,
/// }
///
/// unsafe impl TypeLayout for Foo {
///     const INHABITED: MaybeUninhabited = inhabited::all![u8, u16];
///
///     const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
///         name: ::core::any::type_name::<Self>(),
///         size: ::core::mem::size_of::<Self>(),
///         alignment: ::core::mem::align_of::<Self>(),
///         structure: TypeStructure::Struct {
///             repr: "",
///             fields: &[
///                 Field {
///                     name: "a",
///                     offset: MaybeUninhabited::new::<u8>(::core::mem::offset_of!(Self, a)),
///                     ty: ::core::any::type_name::<u8>(),
///                 },
///                 Field {
///                     name: "b",
///                     offset: MaybeUninhabited::new::<u16>(::core::mem::offset_of!(Self, b)),
///                     ty: ::core::any::type_name::<u16>(),
///                 },
///             ],
///         },
///     };
/// }
/// ```
///
/// Note that if you implement [`TypeLayout`], you should also implement
/// [`typeset::ComputeTypeSet`] for it.
pub unsafe trait TypeLayout: Sized {
    /// Marker for whether the type is
    /// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited) or
    /// [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited).
    const INHABITED: MaybeUninhabited;

    /// Shallow layout of the type.
    const TYPE_LAYOUT: TypeLayoutInfo<'static>;
}

/// Utility trait that provides the deep layout of a type.
pub trait TypeGraphLayout: TypeLayout + typeset::ComputeTypeSet {
    /// Shallow layout of the type.
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: TypeLayout + typeset::ComputeTypeSet> TypeGraphLayout for T {
    const TYPE_GRAPH: TypeLayoutGraph<'static> = TypeLayoutGraph::new::<T>();
}

#[must_use]
/// Compute the number of bytes that this type's [`TypeLayoutGraph`] serialises
/// into.
pub const fn serialised_type_graph_len<T: TypeGraphLayout>() -> usize {
    T::TYPE_GRAPH.serialised_len()
}

#[cfg(feature = "serialize-to-generic-const-array")]
#[must_use]
/// Serialise this type's [`TypeLayoutGraph`] into an array of bytes of length
/// [`serialised_type_graph_len`].
pub const fn serialise_type_graph<T: TypeGraphLayout>() -> [u8; serialised_type_graph_len::<T>()] {
    let mut bytes = [0_u8; serialised_type_graph_len::<T>()];

    T::TYPE_GRAPH.serialise(&mut bytes);

    bytes
}

#[must_use]
/// Hash this type's [`TypeLayoutGraph`] using the provided `seed`.
///
/// The hash is produced over the serialised form of the [`TypeLayoutGraph`], as
/// computed by [`serialise_type_graph`].
pub const fn hash_type_graph<T: TypeGraphLayout>(seed: u64) -> u64 {
    T::TYPE_GRAPH.hash(seed)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", allow(clippy::unsafe_derive_deserialize))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Description of the deep layout of a type.
pub struct TypeLayoutGraph<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    D: Deref<Target = [u8]> = &'a [u8],
    V: Deref<Target = [Variant<'a, F, D>]> = &'a [Variant<'a, F, D>],
    I: Deref<Target = TypeLayoutInfo<'a, F, D, V>> = &'a TypeLayoutInfo<'a, F, D, V>,
    G: Deref<Target = [I]> = &'a [I],
> {
    /// The type's fully-qualified name.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub ty: &'a str,
    /// The list of types that make up the complete graph describing the deep
    /// layout of this type.
    pub tys: G,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Description of the shallow layout of a type.
pub struct TypeLayoutInfo<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    D: Deref<Target = [u8]> = &'a [u8],
    V: Deref<Target = [Variant<'a, F, D>]> = &'a [Variant<'a, F, D>],
> {
    /// The type's fully-qualified name.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub name: &'a str,
    /// The type's size.
    pub size: usize,
    /// The type's minimum alignment.
    pub alignment: usize,
    /// The type's shallow structure.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub structure: TypeStructure<'a, F, D, V>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Description of the shallow structure of a type.
pub enum TypeStructure<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    D: Deref<Target = [u8]> = &'a [u8],
    V: Deref<Target = [Variant<'a, F, D>]> = &'a [Variant<'a, F, D>],
> {
    /// A primitive type, e.g. `()`, `u8`, `*const i32`, `&mut bool`, `[char;
    /// 4]`, or `fn(f32) -> !`.
    Primitive,
    /// A struct-like type, including unit structs, tuple structs, structs, and
    /// tuples.
    Struct {
        /// The string representation of the type's `#[repr(...)]` attributes.
        #[cfg_attr(feature = "serde", serde(borrow))]
        repr: &'a str,
        /// The fields of the struct.
        fields: F,
    },
    /// A union type.
    Union {
        /// The string representation of the type's `#[repr(...)]` attributes.
        #[cfg_attr(feature = "serde", serde(borrow))]
        repr: &'a str,
        /// The fields of the union.
        fields: F,
    },
    /// An enum type.
    Enum {
        /// The string representation of the type's `#[repr(...)]` attributes.
        #[cfg_attr(feature = "serde", serde(borrow))]
        repr: &'a str,
        /// The variants of the union.
        variants: V,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Description of the shallow layout of a variant
pub struct Variant<
    'a,
    F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
    D: Deref<Target = [u8]> = &'a [u8],
> {
    /// The variant's name.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub name: &'a str,
    /// The variant's descriminant, iff the variant is
    /// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub discriminant: MaybeUninhabited<Discriminant<'a, D>>,
    /// The variant's fields.
    pub fields: F,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// Descriptor of the shallow layout of a field.
pub struct Field<'a> {
    /// The field's name.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub name: &'a str,
    /// The field's byte offset, iff the field is
    /// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
    pub offset: MaybeUninhabited<usize>,
    /// The fully-qualified name of the field's type. This is used as a key
    /// inside [`TypeLayoutGraph::tys`] to find the field's type's layout by
    /// its [`TypeLayoutInfo::name`].
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub ty: &'a str,
}

impl TypeLayoutGraph<'static> {
    #[must_use]
    /// Construct the deep type layout descriptor for a type `T`.
    pub const fn new<T: TypeLayout + typeset::ComputeTypeSet>() -> Self {
        Self {
            ty: <T as TypeLayout>::TYPE_LAYOUT.name,
            // SAFETY:
            // - ComputeSet is a sealed trait and its TYS const is always a HList made of only Cons
            //   of &'static TypeLayoutInfo and Empty
            // - Cons is a repr(C) struct with a head followed by a tail, Empty is a zero-sized
            //   repr(C) struct
            // - the HList is layout-equivalent to an array of the same length as ComputeSet::LEN
            // - ComputeSet::TYS provides a static non-dangling reference that we can use to produce
            //   the data pointer for a slice
            tys: unsafe {
                core::slice::from_raw_parts(
                    core::ptr::from_ref(<typeset::TypeSet<T> as typeset::ComputeSet>::TYS).cast(),
                    <typeset::TypeSet<T> as typeset::ComputeSet>::LEN,
                )
            },
        }
    }
}

impl TypeLayoutGraph<'_> {
    #[must_use]
    /// Compute the number of bytes that this [`TypeLayoutGraph`] serialises
    /// into.
    pub const fn serialised_len(&self) -> usize {
        let mut counter = ser::Serialiser::counter(0);
        counter.serialise_type_layout_graph(self);
        let len = counter.cursor();

        let mut last_full_len = len;

        let mut counter = ser::Serialiser::counter(len);
        counter.serialise_usize(last_full_len);
        let mut full_len = counter.cursor();

        while full_len != last_full_len {
            last_full_len = full_len;

            let mut counter = ser::Serialiser::counter(len);
            counter.serialise_usize(last_full_len);
            full_len = counter.cursor();
        }

        full_len
    }

    /// Serialise this [`TypeLayoutGraph`] into the mutable byte slice.
    /// `bytes` must have a length of at least [`Self::serialised_len`].
    ///
    /// Use [`serialise_type_graph`] instead to serialise the
    /// [`TypeLayoutGraph`] of a type `T` into a byte array of the
    /// appropriate length.
    ///
    /// # Panics
    ///
    /// This method panics iff `bytes` has a length of less than
    /// [`Self::serialised_len`].
    pub const fn serialise(&self, bytes: &mut [u8]) {
        let mut writer = ser::Serialiser::writer(bytes, 0);
        writer.serialise_usize(self.serialised_len());
        writer.serialise_type_layout_graph(self);
    }

    #[must_use]
    /// Hash this [`TypeLayoutGraph`] using the provided `hasher`.
    ///
    /// The hash is produced over the serialised form of this
    /// [`TypeLayoutGraph`], as computed by [`Self::serialise`].
    pub const fn hash(&self, seed: u64) -> u64 {
        let mut hasher = ser::Serialiser::hasher(seed);

        hasher.serialise_usize(self.serialised_len());
        hasher.serialise_type_layout_graph(self);

        hasher.hash()
    }
}

impl<
        'a,
        F: Deref<Target = [Field<'a>]> + fmt::Debug,
        D: Deref<Target = [u8]> + fmt::Debug,
        V: Deref<Target = [Variant<'a, F, D>]> + fmt::Debug,
        I: Deref<Target = TypeLayoutInfo<'a, F, D, V>> + fmt::Debug,
        G: Deref<Target = [I]> + fmt::Debug,
    > fmt::Debug for TypeLayoutGraph<'a, F, D, V, I, G>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("TypeLayoutGraph<{}>({:?})", self.ty, self.tys))
    }
}
