# `BitSize` for rust-style enums

Conceptually easy.

- Reserve the relevant number of bits for the discriminant
- Write unpacker for each variant data
- Select unpacker based on discriminant

Problem:

- We would like to have everything be `Struct(uN)`.

Option:

- We could have a `fn variant(&self) -> Option<VariantType>` method for each variant.
- We could separate the "bitfield storage" struct from the actual enum.

```rust
#[bitfield(u32)]
enum FooBar {
    Foo(u7, bool),
    Bar { field1: bool }
    Zab,
}

// Becomes
pub enum FooBarVariants {
    Foo,
    Bar,
    Zab,
}
pub struct FooBarFoo(u32);
impl FooBarFoo {
    pub fn get_0(&self) -> u7 {}
    pub fn get_1(&self) -> bool {}
}
pub struct FooBarBar(u32);
impl FooBarBar {
    pub fn field1(&self) -> bool {}
}

pub struct FooBar(u32);
impl FooBar {
    pub fn variant(&self) -> FooBarVariant {
        // ...
    }

    pub fn foo(&self) -> Option<FooBarFoo> {
        // ...
    }
    pub fn bar(&self) -> Option<FooBarBar> {
        // ...
    }
}