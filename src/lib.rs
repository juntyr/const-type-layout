/*!
[![GitHub CI Status](https://github.com/LPGhatguy/type-layout/workflows/CI/badge.svg)](https://github.com/LPGhatguy/type-layout/actions)
[![type-layout on crates.io](https://img.shields.io/crates/v/type-layout.svg)](https://crates.io/crates/type-layout)
[![type-layout docs](https://img.shields.io/badge/docs-docs.rs-orange.svg)](https://docs.rs/type-layout)

type-layout is a type layout debugging aid, providing a `#[derive]`able trait
that reports:
- The type's name, size, and minimum alignment
- Each field's name, type, offset, and size
- Padding due to alignment requirements

**type-layout currently only functions on structs with named fields.** This is a
temporary limitation.

## Examples

The layout of types is only defined if they're `#[repr(C)]`. This crate works on
non-`#[repr(C)]` types, but their layout is unpredictable.

```rust
use type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C)]
struct Foo {
    a: u8,
    b: u32,
}

println!("{}", Foo::type_layout());
// prints:
// Foo (size 8, alignment 4)
// | Offset | Name      | Size |
// | ------ | --------- | ---- |
// | 0      | a         | 1    |
// | 1      | [padding] | 3    |
// | 4      | b         | 4    |
```

Over-aligned types have trailing padding, which can be a source of bugs in some
FFI scenarios:

```rust
use type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C, align(128))]
struct OverAligned {
    value: u8,
}

println!("{}", OverAligned::type_layout());
// prints:
// OverAligned (size 128, alignment 128)
// | Offset | Name      | Size |
// | ------ | --------- | ---- |
// | 0      | value     | 1    |
// | 1      | [padding] | 127  |
```

## Minimum Supported Rust Version (MSRV)

type-layout supports Rust 1.34.1 and newer. Until type-layout reaches 1.0,
changes to the MSRV will require major version bumps. After 1.0, MSRV changes
will only require minor version bumps, but will need significant justification.
*/

#![deny(clippy::pedantic)]
#![no_std]
#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_ptr_offset)]
#![feature(const_mut_refs)]
#![feature(const_raw_ptr_comparison)]
#![feature(const_trait_impl)]
#![feature(const_fn_trait_bound)]
#![feature(const_panic)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::fmt;
use alloc::str;

pub use type_layout_derive::TypeLayout;

#[doc(hidden)]
pub use memoffset;

pub unsafe trait TypeLayout {
    const TYPE_LAYOUT: TypeLayoutInfo<'static>;
}

macro_rules! impl_primitive_type_layout {
    (impl $ty:ty) => {
        unsafe impl TypeLayout for $ty {
            const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
                name: ::core::any::type_name::<$ty>(),
                size: ::core::mem::size_of::<$ty>(),
                alignment: ::core::mem::align_of::<$ty>(),
                structure: TypeStructure::Primitive,
            };
        }

        unsafe impl const TypeGraph for $ty {
            fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
                graph.insert(&<$ty>::TYPE_LAYOUT);
            }
        }
    };
    ($($ty:ty),*) => {
        $(impl_primitive_type_layout!{impl $ty})*
    };
}

impl_primitive_type_layout! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    char, bool, ()
}

unsafe impl<T: TypeLayout, const N: usize> TypeLayout for [T; N] {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<[T; N]>(),
        size: ::core::mem::size_of::<[T; N]>(),
        alignment: ::core::mem::align_of::<[T; N]>(),
        structure: TypeStructure::Array {
            item: ::core::any::type_name::<T>(),
            len: N,
        },
    };
}

unsafe impl<T: ~const TypeGraph, const N: usize> const TypeGraph for [T; N] {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T> TypeLayout for core::marker::PhantomData<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<core::marker::PhantomData<T>>(),
        size: ::core::mem::size_of::<core::marker::PhantomData<T>>(),
        alignment: ::core::mem::align_of::<core::marker::PhantomData<T>>(),
        structure: TypeStructure::Primitive,
    };
}

unsafe impl<T> const TypeGraph for core::marker::PhantomData<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&Self::TYPE_LAYOUT);
    }
}

unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<&'a T>(),
        size: ::core::mem::size_of::<&'a T>(),
        alignment: ::core::mem::align_of::<&'a T>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: false,
        },
    };
}

unsafe impl<'a, T: ~const TypeGraph + 'static> const TypeGraph for &'a T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<&'a mut T>(),
        size: ::core::mem::size_of::<&'a mut T>(),
        alignment: ::core::mem::align_of::<&'a mut T>(),
        structure: TypeStructure::Reference {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
}

unsafe impl<'a, T: ~const TypeGraph + 'static> const TypeGraph for &'a mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: TypeLayout> TypeLayout for *const T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<*const T>(),
        size: ::core::mem::size_of::<*const T>(),
        alignment: ::core::mem::align_of::<*const T>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: false,
        },
    };
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for *const T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: TypeLayout> TypeLayout for *mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<*mut T>(),
        size: ::core::mem::size_of::<*mut T>(),
        alignment: ::core::mem::align_of::<*mut T>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for *mut T {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: TypeLayout> TypeLayout for alloc::boxed::Box<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<alloc::boxed::Box<T>>(),
        size: ::core::mem::size_of::<alloc::boxed::Box<T>>(),
        alignment: ::core::mem::align_of::<alloc::boxed::Box<T>>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

unsafe impl<T: TypeLayout> TypeLayout for alloc::boxed::Box<[T]> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<alloc::boxed::Box<[T]>>(),
        size: ::core::mem::size_of::<alloc::boxed::Box<[T]>>(),
        alignment: ::core::mem::align_of::<alloc::boxed::Box<[T]>>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<[T]> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
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
    Struct { fields: &'a [Field<'a>] },
    Union { fields: &'a [Field<'a>] },
    Enum { variants: &'a [Variant<'a>] },
    Primitive,
    Array { item: &'a str, len: usize },
    Reference { inner: &'a str, mutability: bool },
    Pointer { inner: &'a str, mutability: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Variant<'a> {
    pub name: &'a str,
    pub discriminant: u128,
    pub fields: &'a [Field<'a>],
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Field<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub ty: &'a str,
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

pub struct TypeLayoutGraph<'a> {
    len: usize,
    tys: [*const TypeLayoutInfo<'a>; TypeLayoutGraph::CAPACITY],
}

impl<'a> TypeLayoutGraph<'a> {
    const CAPACITY: usize = 1024;

    #[must_use]
    pub const fn new() -> Self {
        Self {
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
}

impl<'a> fmt::Debug for TypeLayoutGraph<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "TypeLayoutGraph(")?;

        let mut debug = fmt.debug_list();

        for i in 0..self.len {
            debug.entry(unsafe { &**self.tys.as_ptr().add(i) });
        }

        debug.finish()?;

        write!(fmt, ")")
    }
}

pub unsafe trait TypeGraph: TypeLayout {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>);
}

pub trait TypeGraphLayout: TypeGraph {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
}

impl<T: ~const TypeGraph> const TypeGraphLayout for T {
    const TYPE_GRAPH: TypeLayoutGraph<'static> = {
        let mut graph = TypeLayoutGraph::new();

        <T as TypeGraph>::populate_graph(&mut graph);

        graph
    };
}
