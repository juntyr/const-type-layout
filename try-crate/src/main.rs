#![deny(clippy::pedantic)]
#![feature(cfg_version)]
#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![cfg_attr(not(version("1.57.0")), feature(const_panic))]
#![feature(const_refs_to_cell)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(const_discriminant)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]
#![feature(const_ptr_offset)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use const_type_layout::TypeGraphLayout;
use const_type_layout::TypeLayout;

#[repr(C)]
#[derive(TypeLayout)]
struct Foo1;

#[repr(C)]
#[derive(TypeLayout)]
struct Foo2(u8, u16);

#[repr(C)]
#[derive(TypeLayout)]
struct Foo3 {
    a: u8,
    b: u16,
}

#[repr(C)]
#[derive(TypeLayout)]
struct Foo4<T>(T);

#[repr(C)]
#[derive(TypeLayout)]
union Bar {
    a: u8,
    b: u16,
}

#[derive(TypeLayout)]
enum Never {}

#[derive(TypeLayout)]
enum Single {
    Single,
}

#[repr(C)]
#[repr(u8)]
#[derive(TypeLayout)]
enum Quo<T> {
    Unit,
    Tuple(u8, T),
    Struct { a: T, b: u16, c: T },
}

#[repr(u8, C)]
#[derive(TypeLayout)]
#[layout(free = "Box<List<T>>")]
#[layout(bound = "T: ::const_type_layout::TypeLayout")]
enum List<T> {
    Cons { item: T, next: Box<List<T>> },
    Tail,
}

#[repr(transparent)]
#[derive(TypeLayout)]
pub struct Reference<'r, T: 'r> {
    pointer: *const T,
    reference: std::marker::PhantomData<&'r T>,
}

fn main() {
    println!("{:#?}", Foo1::TYPE_GRAPH);
    println!("{:#?}", Foo2::TYPE_GRAPH);
    println!("{:#?}", Foo3::TYPE_GRAPH);
    println!("{:#?}", Foo4::<u8>::TYPE_GRAPH);

    println!("{:#?}", Bar::TYPE_GRAPH);

    println!("{:#?}", Never::TYPE_GRAPH);
    println!("{:#?}", Single::TYPE_GRAPH);
    println!("{:#?}", Quo::<u32>::TYPE_GRAPH);

    println!("{:#?}", <()>::TYPE_GRAPH);
    println!("{:#?}", <[u32; 3]>::TYPE_GRAPH);
    println!("{:#?}", <std::marker::PhantomData<String>>::TYPE_GRAPH);
    println!("{:#?}", <std::mem::MaybeUninit<Box<i8>>>::TYPE_GRAPH);
    println!("{:#?}", <Box<u8>>::TYPE_GRAPH);
    println!("{:#?}", <Box<[u8]>>::TYPE_GRAPH);

    println!("{:#?}", <Option<std::num::NonZeroU64>>::TYPE_GRAPH);
    println!("{:#?}", <Result<bool, u8>>::TYPE_GRAPH);

    println!("{:#?}", <Reference<i32>>::TYPE_GRAPH);

    println!("{:#?}", <List<u8>>::TYPE_GRAPH);

    let mut ascii_escaped_layout = String::new();
    for b in SERIALISED_LIST_U8_LAYOUT {
        let part: Vec<u8> = std::ascii::escape_default(b).collect();
        ascii_escaped_layout.push_str(std::str::from_utf8(&part).unwrap());
    }
    println!("{}", ascii_escaped_layout);
}

const SERIALISED_LIST_U8_LAYOUT: [u8; const_type_layout::serialised_type_graph_len::<List<u8>>()] =
    const_type_layout::serialise_type_graph::<List<u8>>();
