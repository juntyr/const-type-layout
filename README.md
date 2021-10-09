# const-type-layout &emsp; [![CI Status]][workflow] [![Rust Doc]][docs] [![License Status]][fossa] [![Code Coverage]][codecov] [![Gitpod Ready-to-Code]][gitpod]

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
use type_layout::TypeLayout;

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
use type_layout::TypeLayout;

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
//         repr: "C,align(128)",
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

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
