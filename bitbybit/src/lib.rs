use std::{ops, marker::PhantomData};

pub use arbitrary_int;
use arbitrary_int::UInt;
pub use bitbybit_macros::*;

pub trait BitSize<Data> {
    type Unpacked;

    const BITS: u32;

    fn from_offset<const OFFSET: u32>(data: Data) -> Self::Unpacked;
    fn with_offset<const OFFSET: u32>(self, data: &Data) -> Data;
}

macro_rules! impl_larger {
    ($($int_from:ty => $int_to:ty),+ $(,)?) => {
        $(
            impl<T: BitSize<$int_from>> BitSize<$int_to> for T {
                type Unpacked = T::Unpacked;

                const BITS: u32 = T::BITS;

                #[inline(always)]
                fn from_offset<const OFFSET: u32>(data: $int_to) -> Self::Unpacked {
                    <T as BitSize<$int_from>>::from_offset::<0>((data >> OFFSET) as $int_from)
                }
                #[inline(always)]
                fn with_offset<const OFFSET: u32>(self, data: &$int_to) -> $int_to {
                    let no_offset = <T as BitSize<$int_from>>::with_offset::<0>(self, &0);
                    let no_offset = <$int_to>::from(no_offset);
                    let data = *data;
                    // TODO(bug): overflows
                    let mask = ((1 << Self::BITS) - 1) << OFFSET;
                    (data & !mask) | (no_offset << OFFSET)
                }
            }
        )+
    }
}

macro_rules! impl_bit_size {
    ($($int_ty:ty),+ $(,)?) => {
        $(
            impl BitSize<Self> for $int_ty {
                type Unpacked = Self;

                const BITS: u32 = <$int_ty>::BITS;

                #[inline(always)]
                fn from_offset<const OFFSET: u32>(data: Self) -> Self {
                    assert_eq!(OFFSET, 0);
                    data
                }

                #[inline(always)]
                fn with_offset<const OFFSET: u32>(self, _: &Self) -> Self {
                    assert_eq!(OFFSET, 0);
                    self
                }
            }
        )+
    }
}

macro_rules! impl_unint_bit_size {
    ($($int_ty:ty),+ $(,)?) => {
        $(
            impl<const BITS: usize> BitSize<$int_ty> for UInt<$int_ty, BITS> {
                type Unpacked = Self;

                const BITS: u32 = BITS as u32;

                #[inline(always)]
                fn from_offset<const OFFSET: u32>(data: $int_ty) -> Self {
                    Self::new((data >> OFFSET) & ((1 << BITS) - 1))
                }
                #[inline(always)]
                fn with_offset<const OFFSET: u32>(self, data: &$int_ty) -> $int_ty {
                    let offset = self.value() << OFFSET;
                    (*data & !(((1 << BITS) - 1) << OFFSET)) | offset
                }
            }
        )+
    }
}

macro_rules! impl_tuple_bitsize {
    ($( [$( ($T:ident, $t:ident) )*] )* ) => {
        $(
            impl<D: ops::Shr<u32, Output = Self>, $( $T: BitSize<D>, )* > BitSize<D> for TupleStore<( $( $T, )* )> {
                type Unpacked = Self;

                const BITS: u32 = 0 $( + $T::BITS )*;

                #[inline(always)]
                fn from_offset<const OFFSET: u32>(data: D) -> Self::Unpacked {
                    let mut data = data >> OFFSET;
                    $(
                        let $t = $T::from_offset::<0>(data);
                        data = data >> $T::BITS;
                    )*
                    ( $( $t, )* )
                }

                #[inline(always)]
                fn with_offset<const OFFSET: u32>(self, data: &D) -> D {
                    let mut data = *data;
                    let ( $( $t, )* ) = self;
                    $(
                        let data = $T::with_offset::<OFFSET>($t, &data);
                        data = data >> $T::BITS;
                        let $t = $T::from_offset::<0>(data);
                        data = data >> $T::BITS;
                    )*
                }
            }
        )+
    }
}

impl<D, T: BitSize<D>> BitSize<D> for Option<T> {
    type Unpacked = Self;

    const BITS: u32 = <(bool, T)>::BITS;

    fn from_offset<const OFFSET: u32>(data: D) -> Self::Unpacked {
        let bool_struct = <(bool, T)>::from_offset::<OFFSET>(data);
        bool_struct.0.then_some(bool_struct.1)
    }

    fn with_offset<const OFFSET: u32>(self, data: &D) -> D {
        let mut data = <bool as BitSize<D>>::with_offset::<OFFSET>(self.is_some(), data);
        if let Some(value) = self {
            data = <T as BitSize<D>>::with_offset::<{OFFSET + 1}>(value, &data);
        }
        data
    }
}
impl BitSize<u8> for bool {
    type Unpacked = Self;

    const BITS: u32 = 1;

    #[inline(always)]
    fn from_offset<const OFFSET: u32>(data: u8) -> Self {
        (data & 1 << OFFSET) == 1 << OFFSET
    }
    #[inline(always)]
    fn with_offset<const OFFSET: u32>(self, data: &u8) -> u8 {
        *data | 1 << OFFSET
    }
}
impl_tuple_bitsize!{
    [(T0, t0)]
    [(T0, t0) (T1, t1)]
    [(T0, t0) (T1, t1) (T2, t2)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)(T4, t4)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)(T4, t4)(T5, t5)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)(T4, t4)(T5, t5)(T6, t6)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)(T4, t4)(T5, t5)(T6, t6)(T7, t7)]
    [(T0, t0) (T1, t1) (T2, t2)(T3, t3)(T4, t4)(T5, t5)(T6, t6)(T7, t7)(T8, t8)]
};
impl_bit_size![u8, u16, u32, u64, u128];
impl_unint_bit_size![u8, u16, u32, u64, u128];
impl_larger! {
    u8 => u16,
    u16 => u32,
    u32 => u64,
    u64 => u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary_int::*;

    #[bitenum(u2, exhaustive = true, crate_path = crate)]
    #[derive(PartialEq, Debug)]
    enum Foo {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }
    #[bitfield(storage_type = u16, crate_path = crate)]
    struct MyStruct {
        f1: u7,
        f6: Foo,
    }
    #[test]
    fn foo() {
        let s = MyStruct(0).with_f6(Foo::D);
        assert_eq!(s.f6(), Foo::D)
    }
}
