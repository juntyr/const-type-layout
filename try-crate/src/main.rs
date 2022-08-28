#![deny(clippy::pedantic)]
#![feature(cfg_version)]
#![feature(const_type_name)]
#![feature(const_refs_to_cell)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![cfg_attr(not(version("1.61.0")), feature(const_fn_trait_bound))]
#![cfg_attr(not(version("1.61.0")), feature(const_ptr_offset))]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{borrow::Cow, ops::Deref};

use const_type_layout::{TypeGraphLayout, TypeLayout};

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

#[allow(clippy::empty_enum)]
#[derive(TypeLayout)]
enum Never {}

#[derive(TypeLayout)]
enum Single {
    Single,
}

#[derive(TypeLayout)]
enum Double {
    A = 2,
    B = 3,
}

#[derive(TypeLayout)]
enum WithDouble {
    A(Double),
}

#[repr(C)]
#[repr(u8)]
#[derive(TypeLayout)]
enum Quo<T> {
    Unit,
    Tuple(u8, T),
    Struct { a: T, b: u16, c: T },
}

// TODO: allow annotating any variant as the base case (same for unions)

#[repr(u8, C)]
#[derive(TypeLayout)]
enum List<T> {
    Tail,
    Cons { item: T, next: Box<List<T>> },
}

#[repr(transparent)]
#[derive(TypeLayout)]
pub struct Reference<'r, T: 'r> {
    pointer: *const T,
    reference: std::marker::PhantomData<&'r T>,
}

#[repr(transparent)]
#[derive(TypeLayout)]
pub struct MutReference<'r, T: 'r> {
    pointer: *mut T,
    reference: std::marker::PhantomData<&'r mut T>,
}

#[derive(TypeLayout)]
pub struct Referencing<'r, T: 'r> {
    c: &'r T,
    m: &'r mut T,
}

#[derive(TypeLayout)]
#[layout(free = "T")]
pub struct MyPhantomData<T> {
    marker: std::marker::PhantomData<T>,
}

fn main() {
    println!("{:#?}", Foo1::TYPE_GRAPH);
    println!("{:#?}", Foo2::TYPE_GRAPH);
    println!("{:#?}", Foo3::TYPE_GRAPH);
    println!("{:#?}", Foo4::<u8>::TYPE_GRAPH);

    println!("{:#?}", Bar::TYPE_GRAPH);

    println!("{:#?}", Never::TYPE_GRAPH);
    println!("{:#?}", Single::TYPE_GRAPH);
    println!("{:#?}", Double::TYPE_GRAPH);
    println!("{:#?}", WithDouble::TYPE_GRAPH);
    println!("{:#?}", Quo::<u32>::TYPE_GRAPH);

    println!("{:#?}", <()>::TYPE_GRAPH);
    println!("{:#?}", <[u32; 3]>::TYPE_GRAPH);
    println!("{:#?}", <std::mem::MaybeUninit<Box<i8>>>::TYPE_GRAPH);
    println!("{:#?}", <Box<u8>>::TYPE_GRAPH);
    println!("{:#?}", <Box<[u8]>>::TYPE_GRAPH);
    println!("{:#?}", <Box<&'static u8>>::TYPE_GRAPH);

    println!("{:#?}", <std::marker::PhantomData<bool>>::TYPE_GRAPH);
    println!("{:#?}", <std::marker::PhantomData<String>>::TYPE_GRAPH);
    println!("{:#?}", <MyPhantomData<bool>>::TYPE_GRAPH);
    println!("{:#?}", <MyPhantomData<String>>::TYPE_GRAPH);

    println!("{:#?}", <Option<std::num::NonZeroU64>>::TYPE_GRAPH);
    println!("{:#?}", <Result<bool, u8>>::TYPE_GRAPH);

    println!("{:#?}", <std::convert::Infallible>::TYPE_GRAPH);
    println!("{:#?}", <!>::TYPE_GRAPH);

    // TODO: will require optional uninits to represent uninhabited values
    // println!("{:#?}", <Option<std::convert::Infallible>>::TYPE_GRAPH);
    // println!("{:#?}", <Result<u8, std::convert::Infallible>>::TYPE_GRAPH);
    // println!("{:#?}", <Result<std::convert::Infallible, u8>>::TYPE_GRAPH);
    // println!("{:#?}", <Result<std::convert::Infallible,
    // std::convert::Infallible>>::TYPE_GRAPH);

    println!("{:#?}", <*const u8>::TYPE_GRAPH);
    println!("{:#?}", <*mut u8>::TYPE_GRAPH);
    println!("{:#?}", <&u8>::TYPE_GRAPH);
    println!("{:#?}", <&mut u8>::TYPE_GRAPH);

    println!("{:#?}", <Reference<i32>>::TYPE_GRAPH);
    println!("{:#?}", <MutReference<u32>>::TYPE_GRAPH);
    println!("{:#?}", <Referencing<&'static u8>>::TYPE_GRAPH);

    non_static_ref(&0);

    println!("{:#?}", <List<u8>>::TYPE_GRAPH);

    let mut ascii_escaped_layout = String::new();
    for b in SERIALISED_LIST_U8_LAYOUT {
        let part: Vec<u8> = std::ascii::escape_default(b).collect();
        ascii_escaped_layout.push_str(std::str::from_utf8(&part).unwrap());
    }
    println!("{}", ascii_escaped_layout);

    let ron_layout = ron::to_string(&<List<u8>>::TYPE_GRAPH).unwrap();
    println!("{}", ron_layout);
}

fn non_static_ref<'a>(_val: &'a u128) {
    println!("{:#?}", <Referencing<&'a u8>>::TYPE_GRAPH);
}

const SERIALISED_LIST_U8_LAYOUT: [u8; const_type_layout::serialised_type_graph_len::<List<u8>>()] =
    const_type_layout::serialise_type_graph::<List<u8>>();

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
struct DerefCow<'a, T: Clone + Deref>(Cow<'a, T>);

impl<'a, T: Clone + Deref> Deref for DerefCow<'a, T> {
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
