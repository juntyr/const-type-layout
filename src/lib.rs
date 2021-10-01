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

#[doc(hidden)]
pub extern crate alloc;

use alloc::fmt::{self, Display};
use alloc::str;
use alloc::vec::Vec;

pub use type_layout_derive::TypeLayout;

#[doc(hidden)]
pub use memoffset;

pub trait TypeLayout {
    const TYPE_LAYOUT: TypeLayoutInfo<'static>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
//#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeLayoutInfo<'a> {
    pub name: &'a str,
    pub size: usize,
    pub alignment: usize,
    pub structure: TypeStructure<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
//#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeStructure<'a> {
    Struct { fields: &'a [Field<'a>] },
    Union { fields: &'a [Field<'a>] },
    Enum { variants: &'a [Variant<'a>] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
//#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Variant<'a> {
    pub name: &'a str,
    pub discriminant: usize,
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

#[derive(Clone, Debug, PartialEq, Eq)]
//#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Field<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub offset: usize,
    pub size: usize,
    pub alignment: usize,
}

impl<'a> Ord for Field<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (
            &self.offset,
            &self.size,
            &self.alignment,
            &self.name,
            &self.ty,
        )
            .cmp(&(
                &other.offset,
                &other.size,
                &other.alignment,
                &other.name,
                &other.ty,
            ))
    }
}

impl<'a> PartialOrd for Field<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> fmt::Display for TypeLayoutInfo<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "{} {} (size {}, alignment {})",
            match self.structure {
                TypeStructure::Struct { .. } => "STRUCT",
                TypeStructure::Union { .. } => "UNION",
                TypeStructure::Enum { .. } => "ENUM",
            },
            self.name,
            self.size,
            self.alignment
        )?;

        match self.structure {
            TypeStructure::Struct { fields } | TypeStructure::Union { fields } => {
                format_fields(self.size, fields, formatter)?;
            }
            TypeStructure::Enum { variants } => {
                if variants.is_empty() {
                    return writeln!(formatter, "  never");
                }

                let mut variants = Vec::from(variants);
                variants.sort();

                for variant in variants {
                    writeln!(formatter, "- {} @ {}:", variant.discriminant, variant.name)?;

                    format_fields(self.size, variant.fields, formatter)?;
                }
            }
        }

        Ok(())
    }
}

fn format_fields(
    type_size: usize,
    fields: &[Field],
    formatter: &mut fmt::Formatter,
) -> fmt::Result {
    let mut field_rows = Vec::with_capacity(fields.len());

    let mut sorted_fields = Vec::from(fields);
    sorted_fields.sort();

    let mut prior_field_end = 0;

    for field in sorted_fields {
        if field.offset > prior_field_end {
            field_rows.push(Row {
                offset: prior_field_end,
                name: "[padding]",
                ty: "[padding]",
                size: field.offset - prior_field_end,
                alignment: 0,
            });
        }

        field_rows.push(Row {
            offset: field.offset,
            name: field.name,
            ty: field.ty,
            size: field.size,
            alignment: field.alignment,
        });

        prior_field_end = field.offset + field.size;
    }

    if type_size > prior_field_end {
        field_rows.push(Row {
            offset: prior_field_end,
            name: "[padding]",
            ty: "[padding]",
            size: type_size - prior_field_end,
            alignment: 0,
        });
    }

    if field_rows.is_empty() {
        return writeln!(formatter, "  unit");
    }

    let longest_name = "Name".len().max(
        field_rows
            .iter()
            .map(|row| row.name.len())
            .max()
            .unwrap_or(0),
    );

    let longest_type = "Type"
        .len()
        .max(field_rows.iter().map(|row| row.ty.len()).max().unwrap_or(0));

    let widths = RowWidths {
        offset: "Offset".len(),
        name: longest_name,
        ty: longest_type,
        size: "Size".len(),
        alignment: "Alignment".len(),
    };

    write_row(
        formatter,
        widths,
        &Row {
            offset: "Offset",
            name: "Name",
            ty: "Type",
            size: "Size",
            alignment: "Alignment",
        },
    )?;

    write_row(
        formatter,
        widths,
        &Row {
            offset: "------",
            name: str::repeat("-", longest_name),
            ty: str::repeat("-", longest_type),
            size: "----",
            alignment: "---------",
        },
    )?;

    for row in field_rows {
        write_row(formatter, widths, &row)?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct RowWidths {
    offset: usize,
    name: usize,
    ty: usize,
    size: usize,
    alignment: usize,
}

struct Row<O, N, T, S, A> {
    offset: O,
    name: N,
    ty: T,
    size: S,
    alignment: A,
}

fn write_row<O: Display, N: Display, T: Display, S: Display, A: Display>(
    formatter: &mut fmt::Formatter,
    widths: RowWidths,
    row: &Row<O, N, T, S, A>,
) -> fmt::Result {
    writeln!(
        formatter,
        "| {:<offset_width$} | {:<name_width$} | {:<type_width$} | {:<size_width$} | {:<alignment_width$} |",
        row.offset,
        row.name,
        row.ty,
        row.size,
        row.alignment,
        offset_width = widths.offset,
        name_width = widths.name,
        type_width = widths.ty,
        size_width = widths.size,
        alignment_width = widths.alignment,
    )
}
