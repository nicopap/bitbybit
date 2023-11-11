# Implement `BitSize` for tuples

Difficulty:

- In the `with_offset` impl, we need to get something with different offsets,
  can't do this in `const` context.

Workaround:

Only define tuple `BitSize` impls for specific storage types, so that we can
do math on the stoarge type.

Also we should minimize the size of supported tuples, as we have combinatorial
complexity already with the concrete storage types. And in any case, it's
pretty bad to bulk pack/unpack the tuples.