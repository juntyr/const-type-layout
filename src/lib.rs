/*!
[![CI Status]][workflow] [![Rust Doc]][docs] [![License Status]][fossa] [![Code Coverage]][codecov] [![Gitpod Ready-to-Code]][gitpod]

[CI Status]: https://img.shields.io/github/workflow/status/MomoLangenstein/const-type-layout/CI/main?label=CI
[workflow]: https://github.com/MomoLangenstein/const-type-layout/actions/workflows/ci.yml?query=branch%3Amain

[Rust Doc]: https://img.shields.io/badge/docs-main-blue
[docs]: https://momolangenstein.github.io/const-type-layout/const_type_layout

[License Status]: https://app.fossa.com/api/projects/git%2Bgithub.com%2FMomoLangenstein%2Fconst-type-layout.svg?type=shield
[fossa]: https://app.fossa.com/projects/git%2Bgithub.com%2FMomoLangenstein%2Fconst-type-layout?ref=badge_shield

[Code Coverage]: https://img.shields.io/codecov/c/github/MomoLangenstein/const-type-layout?token=J39WVBIMZX
[codecov]: https://codecov.io/gh/MomoLangenstein/const-type-layout

[Gitpod Ready-to-Code]: https://img.shields.io/badge/Gitpod-ready-blue?logo=gitpod
[gitpod]: https://gitpod.io/#https://github.com/MomoLangenstein/const-type-layout

`const-type-layout` is a type layout comparison aid, providing a `#[derive]`able `TypeLayout` trait
that reports:
- The type's name, size, and minimum alignment
- The type's structure, i.e. struct vs. union vs. enum
- Each field's name and offset
- Each variant's name and discriminant

Through the auto-implemented `TypeGraphLayout` trait, the deep type layout is also reported as a graph.

This crate heavily builds on the original runtime [type-layout](https://github.com/LPGhatguy/type-layout) crate by Lucien Greathouse.

## Examples

The layout of types is only defined if they're `#[repr(C)]`. This crate works on
non-`#[repr(C)]` types, but their layout is unpredictable.

```rust
# #![feature(cfg_version)]
# #![feature(const_type_name)]
# #![feature(const_raw_ptr_deref)]
# #![feature(const_maybe_uninit_as_ptr)]
# #![feature(const_ptr_offset_from)]
# #![cfg_attr(not(version("1.57.0")), feature(const_panic))]
# #![feature(const_refs_to_cell)]
# #![feature(const_maybe_uninit_assume_init)]
# #![feature(const_discriminant)]
# #![feature(const_trait_impl)]
# #![feature(const_mut_refs)]
# #![feature(const_fn_trait_bound)]
# #![allow(incomplete_features)]
# #![feature(generic_const_exprs)]

use const_type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C)]
struct Foo {
    a: u8,
    b: u32,
}

println!("{:#?}", Foo::TYPE_LAYOUT);
// prints:
//
// TypeLayoutInfo {
//     name: "Foo",
//     size: 8,
//     alignment: 4,
//     structure: Struct {
//         repr: "C",
//         fields: [
//             Field {
//                 name: "a",
//                 offset: 0,
//                 ty: "u8",
//             },
//             Field {
//                 name: "b",
//                 offset: 4,
//                 ty: "u32",
//             },
//         ],
//     },
// }
```

Over-aligned types have trailing padding, which can be a source of bugs in some
FFI scenarios:

```rust
# #![feature(cfg_version)]
# #![feature(const_type_name)]
# #![feature(const_raw_ptr_deref)]
# #![feature(const_maybe_uninit_as_ptr)]
# #![feature(const_ptr_offset_from)]
# #![cfg_attr(not(version("1.57.0")), feature(const_panic))]
# #![feature(const_refs_to_cell)]
# #![feature(const_maybe_uninit_assume_init)]
# #![feature(const_discriminant)]
# #![feature(const_trait_impl)]
# #![feature(const_mut_refs)]
# #![feature(const_fn_trait_bound)]
# #![allow(incomplete_features)]
# #![feature(generic_const_exprs)]

use const_type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C, align(128))]
struct OverAligned {
    value: u8,
}

println!("{:#?}", OverAligned::TYPE_LAYOUT);
// prints:
//
// TypeLayoutInfo {
//     name: "OverAligned",
//     size: 128,
//     alignment: 128,
//     structure: Struct {
//         repr: "C",
//         fields: [
//             Field {
//                 name: "value",
//                 offset: 0,
//                 ty: "u8",
//             },
//         ],
//     },
// }
```
*/

#![deny(clippy::pedantic)]
#![no_std]
#![feature(cfg_version)]
#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_ptr_offset)]
#![feature(const_mut_refs)]
#![feature(const_raw_ptr_comparison)]
#![feature(const_trait_impl)]
#![feature(const_fn_trait_bound)]
#![cfg_attr(not(version("1.57.0")), feature(const_panic))]
#![feature(cfg_target_has_atomic)]
#![feature(const_discriminant)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(const_ptr_offset_from)]
#![feature(const_refs_to_cell)]
#![feature(const_maybe_uninit_as_ptr)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::fmt;

pub use const_type_layout_derive::TypeLayout;

mod impls;
mod ser;

pub unsafe trait TypeLayout: Sized {
    const TYPE_LAYOUT: TypeLayoutInfo<'static>;
}

pub unsafe trait TypeGraph: TypeLayout {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>);
}

pub trait TypeGraphLayout: TypeGraph {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: ~const TypeGraph> const TypeGraphLayout for T {
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

pub struct TypeLayoutGraph<'a> {
    ty: &'a str,
    len: usize,
    tys: [*const TypeLayoutInfo<'a>; TypeLayoutGraph::CAPACITY],
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct TypeLayoutInfo<'a> {
    pub name: &'a str,
    pub size: usize,
    pub alignment: usize,
    pub structure: TypeStructure<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub enum TypeStructure<'a> {
    Struct {
        repr: &'a str,
        fields: &'a [Field<'a>],
    },
    Union {
        repr: &'a str,
        fields: &'a [Field<'a>],
    },
    Enum {
        repr: &'a str,
        variants: &'a [Variant<'a>],
    },
    Primitive,
    Array {
        item: &'a str,
        len: usize,
    },
    Reference {
        inner: &'a str,
        mutability: bool,
    },
    Pointer {
        inner: &'a str,
        mutability: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Variant<'a> {
    pub name: &'a str,
    pub discriminant: Discriminant<'a>,
    pub fields: &'a [Field<'a>],
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Discriminant<'a> {
    pub big_endian_bytes: &'a [u8],
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Field<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub ty: &'a str,
}

impl<'a> TypeLayoutGraph<'a> {
    const CAPACITY: usize = 1024;

    #[must_use]
    pub const fn new<T: TypeLayout>() -> Self {
        Self {
            ty: <T as TypeLayout>::TYPE_LAYOUT.name,
            len: 0,
            tys: [core::ptr::null(); Self::CAPACITY],
        }
    }

    /// # Panics
    ///
    /// Panics iff the graph is not large enough to store the extra `ty`.
    pub const fn insert(&mut self, ty: &'a TypeLayoutInfo<'a>) -> bool {
        let ty_name_bytes = ty.name.as_bytes();

        let mut i = 0;

        while i < self.len {
            let cached_ty_name_bytes = unsafe { &*self.tys[i] }.name.as_bytes();

            // The type names can only be equal if they are the same length
            if ty_name_bytes.len() == cached_ty_name_bytes.len() {
                let mut j = 0;

                while j < ty_name_bytes.len() {
                    // Break early, i.e. j < ty_name_bytes.len(),
                    //  if the type names do NOT match
                    if ty_name_bytes[i] != cached_ty_name_bytes[i] {
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

        if i >= self.tys.len() {
            panic!("TypeLayoutGraph is not large enough for this complex type.");
        }

        self.tys[i] = ty;
        self.len += 1;

        true
    }

    #[must_use]
    pub const fn serialised_len(&self) -> usize {
        let len = ser::serialised_type_layout_graph_len(0, self);

        let mut last_full_len = len;
        let mut full_len = ser::serialised_usize_len(len, last_full_len);

        while full_len != last_full_len {
            last_full_len = full_len;
            full_len = ser::serialised_usize_len(len, last_full_len);
        }

        full_len
    }

    pub const fn serialise(&self, bytes: &mut [u8]) {
        let from = ser::serialise_usize(bytes, 0, self.serialised_len());

        ser::serialise_type_layout_graph(bytes, from, self);
    }
}

impl<'a> fmt::Debug for TypeLayoutGraph<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "TypeLayoutGraph<{}>(", self.ty)?;

        let mut debug = fmt.debug_list();

        for i in 0..self.len {
            debug.entry(unsafe { &**self.tys.as_ptr().add(i) });
        }

        debug.finish()?;

        write!(fmt, ")")
    }
}

impl<'a> Ord for Variant<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.discriminant, &self.name, &self.fields).cmp(&(
            &other.discriminant,
            &other.name,
            &other.fields,
        ))
    }
}

impl<'a> PartialOrd for Variant<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> fmt::Debug for Discriminant<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "0x")?;

        let mut is_zero = true;

        for byte in self.big_endian_bytes.iter().copied() {
            if byte != 0_u8 {
                is_zero = false;

                write!(fmt, "{:x}", byte)?;
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
