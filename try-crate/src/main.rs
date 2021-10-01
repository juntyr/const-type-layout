use type_layout::TypeLayout;

#[repr(C)]
#[derive(TypeLayout)]
struct Foo(u8, u16);

#[repr(C)]
#[derive(TypeLayout)]
union Bar {
    a: u8,
    b: u16,
}

#[derive(TypeLayout)]
enum Quo<T> {
    None,
    Some { a: T, b: u16, c: T },
}

fn main() {
    println!("{}", Foo::type_layout());
    println!("{}", Bar::type_layout());
    // TODO: Currently all offsets after the enum
    println!("{}", Quo::<std::num::NonZeroU32>::type_layout());
}
