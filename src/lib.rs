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

#[doc(hidden)]
pub extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::fmt;
use alloc::str;
use core::cell::UnsafeCell;

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
            item: &T::TYPE_LAYOUT,
            len: N,
        },
    };
}

unsafe impl<T> TypeLayout for core::marker::PhantomData<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<core::marker::PhantomData<T>>(),
        size: ::core::mem::size_of::<core::marker::PhantomData<T>>(),
        alignment: ::core::mem::align_of::<core::marker::PhantomData<T>>(),
        structure: TypeStructure::Primitive,
    };
}

/*unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<&'a T>(),
        size: ::core::mem::size_of::<&'a T>(),
        alignment: ::core::mem::align_of::<&'a T>(),
        structure: TypeStructure::Reference { inner: &T::TYPE_LAYOUT, mutability: false },
    };
}

unsafe impl<'a, T: TypeLayout + 'static> TypeLayout for &'a mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<&'a mut T>(),
        size: ::core::mem::size_of::<&'a mut T>(),
        alignment: ::core::mem::align_of::<&'a mut T>(),
        structure: TypeStructure::Reference { inner: &T::TYPE_LAYOUT, mutability: true },
    };
}*/

unsafe impl<T: TypeLayout> TypeLayout for *const T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<*const T>(),
        size: ::core::mem::size_of::<*const T>(),
        alignment: ::core::mem::align_of::<*const T>(),
        structure: TypeStructure::Pointer {
            inner: &T::TYPE_LAYOUT,
            mutability: false,
        },
    };
}

unsafe impl<T: TypeLayout> TypeLayout for *mut T {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<*mut T>(),
        size: ::core::mem::size_of::<*mut T>(),
        alignment: ::core::mem::align_of::<*mut T>(),
        structure: TypeStructure::Pointer {
            inner: &T::TYPE_LAYOUT,
            mutability: true,
        },
    };
}

unsafe impl<T: TypeLayout> TypeLayout for alloc::boxed::Box<T> {
    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<alloc::boxed::Box<T>>(),
        size: ::core::mem::size_of::<alloc::boxed::Box<T>>(),
        alignment: ::core::mem::align_of::<alloc::boxed::Box<T>>(),
        structure: TypeStructure::Pointer {
            inner: &T::TYPE_LAYOUT,
            mutability: true,
        },
    };
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct TypeLayoutInfo<'a> {
    pub name: &'a str,
    pub size: usize,
    pub alignment: usize,
    pub structure: TypeStructure<'a>,
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub enum TypeStructure<'a> {
    Struct {
        fields: &'a [Field<'a>],
    },
    Union {
        fields: &'a [Field<'a>],
    },
    Enum {
        variants: &'a [Variant<'a>],
    },
    Primitive,
    Array {
        item: &'a TypeLayoutInfo<'a>,
        len: usize,
    },
    Reference {
        inner: &'a TypeLayoutInfo<'a>,
        mutability: bool,
    },
    Pointer {
        inner: &'a TypeLayoutInfo<'a>,
        mutability: bool,
    },
}

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Field<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub ty: &'a TypeLayoutInfo<'a>,
}

impl<'a> PartialEq for Field<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.offset == other.offset && core::ptr::eq(self.ty, other.ty)
    }
}

impl<'a> Eq for Field<'a> {}

impl<'a> Ord for Field<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (
            &self.offset,
            &self.ty.size,
            &self.ty.alignment,
            &self.name,
            &self.ty.name,
        )
            .cmp(&(
                &other.offset,
                &other.ty.size,
                &other.ty.alignment,
                &other.name,
                &other.ty.name,
            ))
    }
}

impl<'a> PartialOrd for Field<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TypeLayoutGraph<'a> {
    len: usize,
    tys: [*const TypeLayoutInfo<'a>; 1024],
}

impl<'a> TypeLayoutGraph<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self { len: 0, tys: [core::ptr::null(); 1024] }
    }

    pub const fn insert(&mut self, ty: &'a TypeLayoutInfo<'a>) -> bool {
        let mut i = 0;

        while i < self.len {
            if unsafe { *self.tys.as_ptr().add(i) }.guaranteed_eq(ty) {
                return false;
            }

            i += 1;
        }

        let item = unsafe { &mut *self.tys.as_mut_ptr().add(i) };
        *item = ty;

        self.len += 1;

        true
    }
}

impl<'a> fmt::Debug for TypeLayoutGraph<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let cache = UnsafeCell::new(BTreeSet::new());

        write!(fmt, "TypeLayoutGraph(")?;

        let mut debug = fmt.debug_list();

        for i in 0..self.len {
            debug.entry(&TypeLayoutWrapper {
                ty: TypeLayoutTypes::Info(unsafe { &**self.tys.as_ptr().add(i) }), cache: &cache,
            });
        }

        debug.finish()?;

        write!(fmt, ")")
    }
}

trait TypeGraph {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>);
}

impl const TypeGraph for u8 {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        graph.insert(&u8::TYPE_LAYOUT);
    }
}

pub const TEST: TypeLayoutGraph<'static> = {
    let mut graph = TypeLayoutGraph::new();

    u8::populate_graph(&mut graph);

    graph
};

enum TypeLayoutTypes<'a> {
    Info(&'a TypeLayoutInfo<'a>),
    Structure(&'a TypeStructure<'a>),
    Variant(&'a Variant<'a>),
    Field(&'a Field<'a>),
    Variants(&'a [Variant<'a>]),
    Fields(&'a [Field<'a>]),
}

struct RecursiveTypeLayout;

impl fmt::Debug for RecursiveTypeLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("RECURSIVE").finish()
    }
}

impl fmt::Display for RecursiveTypeLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, fmt)
    }
}

struct TypeLayoutWrapper<'a, 'b> {
    ty: TypeLayoutTypes<'a>,
    cache: &'b UnsafeCell<BTreeSet<*const TypeLayoutInfo<'a>>>,
}

#[allow(clippy::too_many_lines)]
impl<'a, 'b> fmt::Debug for TypeLayoutWrapper<'a, 'b> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.ty {
            TypeLayoutTypes::Info(info) => {
                let mut debug = fmt.debug_struct("TypeLayoutInfo");

                debug
                    .field("name", &info.name)
                    .field("size", &info.size)
                    .field("alignment", &info.alignment);

                if unsafe { &mut *self.cache.get() }.insert(*info as *const TypeLayoutInfo<'a>) {
                    debug.field(
                        "structure",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Structure(&info.structure),
                            cache: self.cache,
                        },
                    )
                } else {
                    debug.field("structure", &RecursiveTypeLayout)
                };

                debug.finish()
            }
            TypeLayoutTypes::Structure(structure) => match structure {
                TypeStructure::Primitive => fmt.debug_struct("Primitive").finish(),
                TypeStructure::Struct { fields } => fmt
                    .debug_struct("Struct")
                    .field(
                        "fields",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Fields(fields),
                            cache: self.cache,
                        },
                    )
                    .finish(),
                TypeStructure::Union { fields } => fmt
                    .debug_struct("Union")
                    .field(
                        "fields",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Fields(fields),
                            cache: self.cache,
                        },
                    )
                    .finish(),
                TypeStructure::Enum { variants } => fmt
                    .debug_struct("Enum")
                    .field(
                        "fields",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Variants(variants),
                            cache: self.cache,
                        },
                    )
                    .finish(),
                TypeStructure::Array { len, item } => fmt
                    .debug_struct("Array")
                    .field("len", len)
                    .field(
                        "item",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Info(item),
                            cache: self.cache,
                        },
                    )
                    .finish(),
                TypeStructure::Reference { mutability, inner } => fmt
                    .debug_struct("Reference")
                    .field("mutability", mutability)
                    .field(
                        "item",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Info(inner),
                            cache: self.cache,
                        },
                    )
                    .finish(),
                TypeStructure::Pointer { mutability, inner } => fmt
                    .debug_struct("Pointer")
                    .field("mutability", mutability)
                    .field(
                        "item",
                        &TypeLayoutWrapper {
                            ty: TypeLayoutTypes::Info(inner),
                            cache: self.cache,
                        },
                    )
                    .finish(),
            },
            TypeLayoutTypes::Variant(variant) => fmt
                .debug_struct("Variant")
                .field("name", &variant.name)
                .field("discriminant", &variant.discriminant)
                .field(
                    "fields",
                    &TypeLayoutWrapper {
                        ty: TypeLayoutTypes::Fields(variant.fields),
                        cache: self.cache,
                    },
                )
                .finish(),
            TypeLayoutTypes::Field(field) => fmt
                .debug_struct("Field")
                .field("name", &field.name)
                .field("offset", &field.offset)
                .field(
                    "ty",
                    &TypeLayoutWrapper {
                        ty: TypeLayoutTypes::Info(field.ty),
                        cache: self.cache,
                    },
                )
                .finish(),
            TypeLayoutTypes::Variants(variants) => {
                let mut debug = fmt.debug_list();

                for variant in variants.iter() {
                    debug.entry(&TypeLayoutWrapper {
                        ty: TypeLayoutTypes::Variant(variant),
                        cache: self.cache,
                    });
                }

                debug.finish()
            }
            TypeLayoutTypes::Fields(fields) => {
                let mut debug = fmt.debug_list();

                for field in fields.iter() {
                    debug.entry(&TypeLayoutWrapper {
                        ty: TypeLayoutTypes::Field(field),
                        cache: self.cache,
                    });
                }

                debug.finish()
            }
        }
    }
}

impl<'a, 'b> fmt::Display for TypeLayoutWrapper<'a, 'b> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, fmt)
    }
}

macro_rules! impl_debug_display_for_layout_types {
    (impl $ty:ident => $var:ident) => {
        impl<'a> fmt::Debug for $ty<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                let cache = UnsafeCell::new(BTreeSet::new());

                fmt::Debug::fmt(&TypeLayoutWrapper {
                    ty: TypeLayoutTypes::$var(self), cache: &cache,
                }, fmt)
            }
        }

        impl<'a> fmt::Display for $ty<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                let cache = UnsafeCell::new(BTreeSet::new());

                fmt::Display::fmt(&TypeLayoutWrapper {
                    ty: TypeLayoutTypes::$var(self), cache: &cache,
                }, fmt)
            }
        }
    };
    ($($ty:ident => $var:ident),*) => {
        $(impl_debug_display_for_layout_types!{impl $ty => $var})*
    };
}

impl_debug_display_for_layout_types! {
    TypeLayoutInfo => Info,
    TypeStructure => Structure,
    Variant => Variant,
    Field => Field
}
