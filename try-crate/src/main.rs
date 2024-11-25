#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![feature(const_type_name)]
#![feature(offset_of_enum)]
#![feature(never_type)]
#![feature(sync_unsafe_cell)]
#![feature(exclusive_wrapper)]
#![feature(cfg_version)]
#![cfg_attr(not(version("1.82")), feature(offset_of_nested))]
#![allow(dead_code)]

use const_type_layout::{TypeGraphLayout, TypeLayout};

pub use const_type_layout as ctl;

#[repr(C)]
#[derive(TypeLayout)]
#[layout(crate = "crate::ctl")]
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

#[derive(TypeLayout)]
struct Foo5<A, B> {
    a: (),
    b: (i32,),
    c: (Foo1, Foo2),
    d: (Foo3, Foo4<A>, Foo4<B>),
}

#[repr(C)]
#[derive(TypeLayout)]
union Bar {
    a: u8,
    b: bool,
}

#[repr(C)]
#[derive(TypeLayout)]
union SingleUnion {
    a: u8,
}

#[derive(TypeLayout)]
union RecursiveRef<'a> {
    a: &'a RecursiveRef<'a>,
    b: (),
}

#[derive(TypeLayout)]
union RecursivePtr {
    a: *const RecursivePtr,
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

#[derive(TypeLayout)]
enum NoUnit<T> {
    A(T),
    B(T),
}

#[derive(TypeLayout)]
struct Box<T> {
    pointer: std::ptr::NonNull<T>,
    marker: std::marker::PhantomData<T>,
}

#[repr(u8, C)]
#[derive(TypeLayout)]
enum List<T> {
    Cons { item: T, next: Box<List<T>> },
    Tail,
}

#[derive(TypeLayout)]
enum Tree<T> {
    Node {
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
    Leaf {
        item: T,
    },
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

#[derive(TypeLayout)]
#[repr(transparent)]
pub struct Wrapper(f64);

#[derive(TypeLayout)]
pub struct Bounded<T: std::fmt::Debug + TypeGraphLayout>(T);

fn main() {
    println!("{:#?}", Foo1::TYPE_GRAPH);
    println!("{:#?}", Foo2::TYPE_GRAPH);
    println!("{:#?}", Foo3::TYPE_GRAPH);
    println!("{:#?}", Foo4::<u8>::TYPE_GRAPH);
    println!("{:#?}", Foo5::<u8, i8>::TYPE_GRAPH);
    println!("{:#?}", Foo5::<!, char>::TYPE_GRAPH);
    println!("{:#?}", Foo5::<fn(), unsafe fn(i32) -> bool>::TYPE_GRAPH);
    println!(
        "{:#?}",
        Foo5::<extern "C" fn(), unsafe extern "C" fn(i32) -> bool>::TYPE_GRAPH
    );
    println!(
        "{:#?}",
        Foo4::<unsafe extern "C" fn(i32, !, ...)>::TYPE_GRAPH
    );

    println!("{:#?}", Bar::TYPE_GRAPH);
    println!("{:#?}", SingleUnion::TYPE_GRAPH);
    println!("{:#?}", RecursiveRef::<'static>::TYPE_GRAPH);
    println!("{:#?}", RecursivePtr::TYPE_GRAPH);

    println!("{:#?}", Never::TYPE_GRAPH);
    println!("{:#?}", Single::TYPE_GRAPH);
    println!("{:#?}", Double::TYPE_GRAPH);
    println!("{:#?}", WithDouble::TYPE_GRAPH);
    println!("{:#?}", Quo::<u32>::TYPE_GRAPH);
    println!("{:#?}", NoUnit::<u32>::TYPE_GRAPH);

    println!("{:#?}", <()>::TYPE_GRAPH);
    println!("{:#?}", <(u8,)>::TYPE_GRAPH);
    println!("{:#?}", <(u8, bool)>::TYPE_GRAPH);
    println!("{:#?}", <(u8, bool, !)>::TYPE_GRAPH);
    println!("{:#?}", <[u32; 3]>::TYPE_GRAPH);
    println!("{:#?}", <std::mem::MaybeUninit<Box<i8>>>::TYPE_GRAPH);
    println!("{:#?}", <Box<u8>>::TYPE_GRAPH);
    println!("{:#?}", <Box<&'static u8>>::TYPE_GRAPH);

    println!("{:#?}", <std::marker::PhantomData<bool>>::TYPE_GRAPH);
    println!("{:#?}", <std::marker::PhantomData<String>>::TYPE_GRAPH);
    println!("{:#?}", <MyPhantomData<bool>>::TYPE_GRAPH);
    println!("{:#?}", <MyPhantomData<String>>::TYPE_GRAPH);

    println!("{:#?}", <Option<std::num::NonZeroU64>>::TYPE_GRAPH);
    println!("{:#?}", <Result<bool, u8>>::TYPE_GRAPH);

    println!("{:#?}", <std::cell::SyncUnsafeCell<u8>>::TYPE_GRAPH);
    println!("{:#?}", <std::sync::Exclusive<u8>>::TYPE_GRAPH);

    println!("{:#?}", <std::cmp::Ordering>::TYPE_GRAPH);
    println!(
        "{:#?}",
        <std::mem::Discriminant<std::cmp::Ordering>>::TYPE_GRAPH
    );

    println!("{:#?}", <std::convert::Infallible>::TYPE_GRAPH);
    println!("{:#?}", <!>::TYPE_GRAPH);

    println!("{:#?}", <Option<std::convert::Infallible>>::TYPE_GRAPH);
    println!("{:#?}", <Result<u8, std::convert::Infallible>>::TYPE_GRAPH);
    println!("{:#?}", <Result<std::convert::Infallible, u8>>::TYPE_GRAPH);
    println!(
        "{:#?}",
        <Result<std::convert::Infallible, std::convert::Infallible>>::TYPE_GRAPH
    );

    println!("{:#?}", <*const u8>::TYPE_GRAPH);
    println!("{:#?}", <*mut u8>::TYPE_GRAPH);
    println!("{:#?}", <core::ptr::NonNull<u8>>::TYPE_GRAPH);
    println!("{:#?}", <core::sync::atomic::AtomicPtr<u8>>::TYPE_GRAPH);
    println!("{:#?}", <&u8>::TYPE_GRAPH);
    println!("{:#?}", <&mut u8>::TYPE_GRAPH);

    println!("{:#?}", <Reference<i32>>::TYPE_GRAPH);
    println!("{:#?}", <MutReference<u32>>::TYPE_GRAPH);
    println!("{:#?}", <Referencing<&'static u8>>::TYPE_GRAPH);

    println!("{:#?}", <Wrapper>::TYPE_GRAPH);
    println!("{:#?}", <Bounded<bool>>::TYPE_GRAPH);

    non_static_ref(&0);

    println!("{:#?}", <List<u8>>::TYPE_GRAPH);
    println!("{:#?}", <Tree<u8>>::TYPE_GRAPH);

    // let mut ascii_escaped_layout = String::new();
    // for b in SERIALISED_LIST_U8_LAYOUT {
    //     let part: Vec<u8> = std::ascii::escape_default(b).collect();
    //     ascii_escaped_layout.push_str(std::str::from_utf8(&part).unwrap());
    // }
    // println!("{ascii_escaped_layout}");

    let ron_layout = ron::to_string(&<List<u8>>::TYPE_GRAPH).unwrap();
    println!("{ron_layout}");
}

fn non_static_ref<'a>(_val: &'a u128) {
    println!("{:#?}", <Referencing<&'a u8>>::TYPE_GRAPH);
}

// const SERIALISED_LIST_U8_LAYOUT: [u8; const_type_layout::serialised_type_graph_len::<List<u8>>()] =
//     const_type_layout::serialise_type_graph::<List<u8>>();
