use crate::Bits;
use std::error::Error as TError;

/*pub trait Garbler<B: Bits> {
    type Gate;
    type EncodedGate;
    type EncodedWire;
    type Error: TError;

    fn encode_gate(&self, gate: &Self::Gate) -> Result<Self::EncodedGate, Self::Error>;
    fn encode_wire(&self, wire_index: usize) -> Result<Self::EncodedWire, Self::Error>;
}

pub trait Evaluator<B: Bits, G:Garbler<B>> {
    fn add_encoded_wire(&mut self, encoded_wire:G::EncodedWire) -> Result<(),G::Error>;
}*/
