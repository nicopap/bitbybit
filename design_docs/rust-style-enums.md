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
