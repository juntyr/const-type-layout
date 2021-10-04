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
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::fmt;

pub use type_layout_derive::TypeLayout;

#[doc(hidden)]
pub use memoffset;

pub unsafe trait TypeLayout: Sized {
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

const fn serialise_str(bytes: &mut [u8], from: usize, value: &str) -> usize {
    let value_bytes = value.as_bytes();

    if value_bytes.len() > (u16::MAX as usize) {
        panic!("str is too long to be serialised.");
    }

    let from = serialise_usize(bytes, from, value_bytes.len());

    if (from + value_bytes.len()) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised str.");
    }

    let mut i = 0;

    while i < value_bytes.len() {
        bytes[from + i] = value_bytes[i];

        i += 1;
    }

    from + i
}

const fn serialised_str_len(from: usize, value: &str) -> usize {
    let value_bytes = value.as_bytes();

    let from = serialised_usize_len(from, value_bytes.len());

    from + value_bytes.len()
}

const fn serialise_usize(bytes: &mut [u8], from: usize, value: usize) -> usize {
    if value > (u16::MAX as usize) {
        panic!("usize is too large to be serialised.");
    }

    #[allow(clippy::cast_possible_truncation)]
    let value_bytes = (value as u16).to_be_bytes();

    if (from + value_bytes.len()) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised usize.");
    }

    let mut i = 0;

    while i < value_bytes.len() {
        bytes[from + i] = value_bytes[i];

        i += 1;
    }

    from + i
}

const fn serialised_usize_len(from: usize, _value: usize) -> usize {
    from + core::mem::size_of::<u16>()
}

const fn serialise_byte(bytes: &mut [u8], from: usize, value: u8) -> usize {
    if from >= bytes.len() {
        panic!("bytes is not large enough to contain the serialised byte.");
    }

    bytes[from] = value;

    from + 1
}

const fn serialised_byte_len(from: usize, _value: u8) -> usize {
    from + 1
}

const fn serialise_bool(bytes: &mut [u8], from: usize, value: bool) -> usize {
    if from >= bytes.len() {
        panic!("bytes is not large enough to contain the serialised bool.");
    }

    bytes[from] = if value { b'T' } else { b'F' };

    from + 1
}

const fn serialised_bool_len(from: usize, _value: bool) -> usize {
    from + 1
}

const fn serialise_discriminant<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &Discriminant<'a>,
) -> usize {
    let value_bytes = value.big_endian_bytes;

    let mut leading_zeroes = 0;

    while leading_zeroes < value_bytes.len() {
        if value_bytes[leading_zeroes] != 0_u8 {
            break;
        }

        leading_zeroes += 1;
    }

    if (value_bytes.len() - leading_zeroes) > (u16::MAX as usize) {
        panic!("discriminant is too long to be serialised.");
    }

    let from = serialise_usize(bytes, from, value_bytes.len() - leading_zeroes);

    if (from + value_bytes.len() - leading_zeroes) > bytes.len() {
        panic!("bytes is not large enough to contain the serialised discriminant.");
    }

    let mut i = leading_zeroes;

    while i < value_bytes.len() {
        bytes[from + i - leading_zeroes] = value_bytes[i];

        i += 1;
    }

    from + i - leading_zeroes
}

const fn serialised_discriminant_len(from: usize, value: &Discriminant) -> usize {
    let value_bytes = value.big_endian_bytes;

    let mut leading_zeroes = 0;

    while leading_zeroes < value_bytes.len() {
        if value_bytes[leading_zeroes] != 0_u8 {
            break;
        }

        leading_zeroes += 1;
    }

    let from = serialised_usize_len(from, value_bytes.len() - leading_zeroes);

    from + value_bytes.len() - leading_zeroes
}

const fn serialise_field<'a>(bytes: &mut [u8], from: usize, value: &Field<'a>) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.offset);
    serialise_str(bytes, from, value.ty)
}

const fn serialised_field_len(from: usize, value: &Field) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.offset);
    serialised_str_len(from, value.ty)
}

const fn serialise_fields<'a>(bytes: &mut [u8], from: usize, value: &[Field<'a>]) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_field(bytes, from, &value[i]);

        i += 1;
    }

    from
}

const fn serialised_fields_len(from: usize, value: &[Field]) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_field_len(from, &value[i]);

        i += 1;
    }

    from
}

const fn serialise_variant<'a>(bytes: &mut [u8], from: usize, value: &Variant<'a>) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_discriminant(bytes, from, &value.discriminant);
    serialise_fields(bytes, from, value.fields)
}

const fn serialised_variant_len(from: usize, value: &Variant) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_discriminant_len(from, &value.discriminant);
    serialised_fields_len(from, value.fields)
}

const fn serialise_variants<'a>(bytes: &mut [u8], from: usize, value: &[Variant<'a>]) -> usize {
    let mut from = serialise_usize(bytes, from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialise_variant(bytes, from, &value[i]);

        i += 1;
    }

    from
}

const fn serialised_variants_len(from: usize, value: &[Variant]) -> usize {
    let mut from = serialised_usize_len(from, value.len());

    let mut i = 0;

    while i < value.len() {
        from = serialised_variant_len(from, &value[i]);

        i += 1;
    }

    from
}

const fn serialise_type_structure<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeStructure<'a>,
) -> usize {
    match value {
        TypeStructure::Struct { fields } => {
            let from = serialise_byte(bytes, from, b's');
            serialise_fields(bytes, from, fields)
        }
        TypeStructure::Union { fields } => {
            let from = serialise_byte(bytes, from, b'u');
            serialise_fields(bytes, from, fields)
        }
        TypeStructure::Enum { variants } => {
            let from = serialise_byte(bytes, from, b'e');
            serialise_variants(bytes, from, variants)
        }
        TypeStructure::Primitive => serialise_byte(bytes, from, b'v'),
        TypeStructure::Array { item, len } => {
            let from = serialise_byte(bytes, from, b'a');
            let from = serialise_str(bytes, from, item);
            serialise_usize(bytes, from, *len)
        }
        TypeStructure::Reference { inner, mutability } => {
            let from = serialise_byte(bytes, from, b'r');
            let from = serialise_str(bytes, from, inner);
            serialise_bool(bytes, from, *mutability)
        }
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialise_byte(bytes, from, b'p');
            let from = serialise_str(bytes, from, inner);
            serialise_bool(bytes, from, *mutability)
        }
    }
}

const fn serialised_type_structure_len(from: usize, value: &TypeStructure) -> usize {
    match value {
        TypeStructure::Struct { fields } => {
            let from = serialised_byte_len(from, b's');
            serialised_fields_len(from, fields)
        }
        TypeStructure::Union { fields } => {
            let from = serialised_byte_len(from, b'u');
            serialised_fields_len(from, fields)
        }
        TypeStructure::Enum { variants } => {
            let from = serialised_byte_len(from, b'e');
            serialised_variants_len(from, variants)
        }
        TypeStructure::Primitive => serialised_byte_len(from, b'v'),
        TypeStructure::Array { item, len } => {
            let from = serialised_byte_len(from, b'a');
            let from = serialised_str_len(from, item);
            serialised_usize_len(from, *len)
        }
        TypeStructure::Reference { inner, mutability } => {
            let from = serialised_byte_len(from, b'r');
            let from = serialised_str_len(from, inner);
            serialised_bool_len(from, *mutability)
        }
        TypeStructure::Pointer { inner, mutability } => {
            let from = serialised_byte_len(from, b'p');
            let from = serialised_str_len(from, inner);
            serialised_bool_len(from, *mutability)
        }
    }
}

const fn serialise_type_layout_info<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutInfo<'a>,
) -> usize {
    let from = serialise_str(bytes, from, value.name);
    let from = serialise_usize(bytes, from, value.size);
    let from = serialise_usize(bytes, from, value.alignment);
    serialise_type_structure(bytes, from, &value.structure)
}

const fn serialised_type_layout_info_len(from: usize, value: &TypeLayoutInfo) -> usize {
    let from = serialised_str_len(from, value.name);
    let from = serialised_usize_len(from, value.size);
    let from = serialised_usize_len(from, value.alignment);
    serialised_type_structure_len(from, &value.structure)
}

const fn serialise_type_layout_graph<'a>(
    bytes: &mut [u8],
    from: usize,
    value: &TypeLayoutGraph<'a>,
) -> usize {
    let from = serialise_str(bytes, from, value.ty);

    let mut from = serialise_usize(bytes, from, value.len);

    let mut i = 0;

    while i < value.len {
        from = serialise_type_layout_info(bytes, from, unsafe { &*value.tys[i] });

        i += 1;
    }

    from
}

const fn serialised_type_layout_graph_len(from: usize, value: &TypeLayoutGraph) -> usize {
    let from = serialised_str_len(from, value.ty);

    let mut from = serialised_usize_len(from, value.len);

    let mut i = 0;

    while i < value.len {
        from = serialised_type_layout_info_len(from, unsafe { &*value.tys[i] });

        i += 1;
    }

    from
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
    pub discriminant: Discriminant<'a>,
    pub fields: &'a [Field<'a>],
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, /*serde::Deserialize*/))]
pub struct Discriminant<'a> {
    pub big_endian_bytes: &'a [u8],
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
    ty: &'a str,
    len: usize,
    tys: [*const TypeLayoutInfo<'a>; TypeLayoutGraph::CAPACITY],
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
        serialised_type_layout_graph_len(2, self)
    }

    pub const fn serialise(&self, bytes: &mut [u8]) {
        let len = serialise_type_layout_graph(bytes, 2, self);
        serialise_usize(bytes, 0, len);
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

pub unsafe trait TypeGraph: TypeLayout {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>);
}

pub trait TypeGraphLayout: TypeGraph {
    const TYPE_GRAPH: TypeLayoutGraph<'static>;
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

impl<T: ~const TypeGraph> const TypeGraphLayout for T {
    const TYPE_GRAPH: TypeLayoutGraph<'static> = {
        let mut graph = TypeLayoutGraph::new::<T>();

        <T as TypeGraph>::populate_graph(&mut graph);

        graph
    };
}
