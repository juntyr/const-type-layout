#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_panic)]
#![feature(const_refs_to_cell)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(const_discriminant)]
#![feature(const_transmute_copy)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]

use type_layout::TypeGraphLayout;
use type_layout::TypeLayout;

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

#[derive(TypeLayout)]
enum Quo<T> {
    Unit,
    Tuple(u8, T),
    Struct { a: T, b: u16, c: T },
}

#[derive(TypeLayout)]
enum List<T> {
    Cons { item: T, next: Box<T> },
    Tail,
}

fn main() {
    println!("{:#?}", Foo1::type_graph());
    println!("{:#?}", Foo2::type_graph());
    println!("{:#?}", Foo3::type_graph());
    println!("{:#?}", Foo4::<u8>::type_graph());

    println!("{:#?}", Bar::type_graph());

    println!("{:#?}", Never::type_graph());
    println!("{:#?}", Single::type_graph());
    println!("{:#?}", Quo::<u32>::type_graph());

    println!("{:#?}", <()>::type_graph());
    println!("{:#?}", <[u32; 3]>::type_graph());
    println!("{:#?}", <std::marker::PhantomData<String>>::type_graph());
    println!("{:#?}", <Box<u8>>::type_graph());
    println!("{:#?}", <Box<[u8]>>::type_graph());

    println!("{:#?}", <List<u8>>::type_graph());
}
