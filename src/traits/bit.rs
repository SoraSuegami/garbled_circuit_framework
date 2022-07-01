use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub trait Bit:
    Sized
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Not<Output = Self>
    + Copy
{
    fn from_const_bit(bit: bool) -> Self;
    fn eq(&self, other: &Self) -> Self;
    fn neq(&self, other: &Self) -> Self;
}

pub trait Bits: Bit + From<Self::Element> + From<usize> {
    type Element: Bit;
    fn from_const_bits(bits: &[bool]) -> Self;
    fn to_elements(self) -> Vec<Self::Element>;
    fn len(&self) -> usize;
    fn concat(&self, other: &Self) -> Self;
    fn mux(&self, true_bits: &Self, select_bit: &Self::Element) -> Self;
    fn get_lsb(&self) -> Self::Element;
    fn modify_lsb(&mut self, value: Self::Element);
}
