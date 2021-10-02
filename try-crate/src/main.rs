#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_panic)]
#![feature(const_refs_to_cell)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(const_discriminant)]
#![feature(const_transmute_copy)]

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

// TODO: Recursive types do NOT work yet

#[derive(TypeLayout)]
enum List<T> {
    Cons { item: T, next: Box<T> },
    Tail,
}

fn main() {
    println!("{:#}", Foo1::TYPE_LAYOUT);
    println!("{:#}", Foo2::TYPE_LAYOUT);
    println!("{:#}", Foo3::TYPE_LAYOUT);
    println!("{:#}", Foo4::<u8>::TYPE_LAYOUT);

    println!("{:#}", Bar::TYPE_LAYOUT);

    println!("{:#}", Never::TYPE_LAYOUT);
    println!("{:#}", Single::TYPE_LAYOUT);
    println!("{:#}", Quo::<u32>::TYPE_LAYOUT);

    println!("{:#}", <()>::TYPE_LAYOUT);
    println!("{:#}", <[u32; 3]>::TYPE_LAYOUT);
    println!("{:#}", <std::marker::PhantomData<String>>::TYPE_LAYOUT);

    println!("{:#}", <List<u8>>::TYPE_LAYOUT);
}
