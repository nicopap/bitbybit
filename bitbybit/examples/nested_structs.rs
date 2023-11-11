use bitbybit::{arbitrary_int::*, bitenum, bitfield};

#[bitenum(u2, exhaustive = true)]
#[derive(PartialEq, Debug)]
enum Foo {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}
#[bitfield(storage_type = u16)]
#[derive(Debug)]
struct MyStruct {
    f1: u7,
    f6: Foo,
}

#[bitfield(storage_type = u8)]
#[derive(Debug)]
struct Zst;

#[bitfield(storage_type = u16)]
#[derive(Debug)]
struct NestFields {
    f9: MyStruct,
    _x0: Zst,
    _x1: Zst,
    _x4: Zst,
    _x5: Zst,
    _x6: Zst,
    _x7: Zst,
    _x8: Zst,
    f5: u5,
}
#[inline(never)]
fn print_f(f: Foo) {
    let f = std::hint::black_box(f);
    println!("{f:?}");
}
fn main() {
    let x = NestFields(0)
        .with_f9(MyStruct(0).with_f6(Foo::C))
        .with_f5(u5::new(4));
    let x = std::hint::black_box(x);
    let f = std::hint::black_box(x.f9().f6());
    print_f(f);
}
