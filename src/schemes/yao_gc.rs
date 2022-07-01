use crate::{Bit, Bits, HashFn, HashFnError, PRFError, PRF};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YaoGCError {
    #[error(transparent)]
    HashFn(#[from] HashFnError),
    #[error(transparent)]
    PRF(#[from] PRFError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedWire<B: Bits> {
    pub wire_index: usize,
    false_value: B,
    true_value: B,
    false_color_bit: B::Element,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate {
    pub wire_l: usize,
    pub wire_r: usize,
    pub wire_o: usize,
    pub gate_type: GateType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateType {
    Not,
    Xor,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedGate<B: Bits> {
    pub gate: Gate,
    cts: [B; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Garbler<B: Bits, H: HashFn<B>, R: PRF<B>> {
    num_input: usize,
    fixed_prf_seed: B,
    hash_fn: H,
    prf: R,
}

impl<B: Bits, H: HashFn<B>, R: PRF<B>> Garbler<B, H, R> {
    pub fn encode_wire(&self, wire_index: usize) -> Result<EncodedWire<B>, YaoGCError> {
        let wire_bits = B::from(wire_index);
        let seed = self.fixed_prf_seed.concat(&wire_bits);
        let false_seed = seed.concat(&B::from_const_bit(false));
        let false_value = self.prf.compute(&false_seed)?;
        let true_seed = seed.concat(&B::from_const_bit(true));
        let mut true_value = self.prf.compute(&true_seed)?;
        let false_color_bit = false_value.get_lsb();
        true_value.modify_lsb(!false_color_bit);
        Ok(EncodedWire {
            wire_index,
            false_value,
            true_value,
            false_color_bit,
        })
    }

    pub fn encode_gate(&self, gate: Gate) -> Result<EncodedGate<B>, YaoGCError> {
        let false_bits = B::from_const_bit(false);
        let true_bits = B::from_const_bit(true);
        let encoded_wire_l = self.encode_wire(gate.wire_l)?;
        let l_color_false_bit = false_bits
            .mux(&true_bits, &encoded_wire_l.false_color_bit)
            .get_lsb();
        let (l_color_false, l_color_true) = (
            encoded_wire_l
                .false_value
                .mux(&encoded_wire_l.true_value, &l_color_false_bit),
            encoded_wire_l
                .true_value
                .mux(&encoded_wire_l.false_value, &l_color_false_bit),
        );
        let encoded_wire_r = self.encode_wire(gate.wire_r)?;
        let r_color_false_bit = false_bits
            .mux(&true_bits, &encoded_wire_r.false_color_bit)
            .get_lsb();
        let (r_color_false, r_color_true) = (
            encoded_wire_r
                .false_value
                .mux(&encoded_wire_r.true_value, &r_color_false_bit),
            encoded_wire_r
                .true_value
                .mux(&encoded_wire_r.false_value, &r_color_false_bit),
        );
        let encoded_wire_o = self.encode_wire(gate.wire_o)?;
        let o_false_false = encoded_wire_o.false_value.mux(
            &encoded_wire_o.true_value,
            &Self::lookup_gate(&gate, l_color_false_bit, r_color_false_bit),
        );
        let o_false_true = encoded_wire_o.false_value.mux(
            &encoded_wire_o.true_value,
            &Self::lookup_gate(&gate, l_color_false_bit, !r_color_false_bit),
        );
        let o_true_false = encoded_wire_o.false_value.mux(
            &encoded_wire_o.true_value,
            &Self::lookup_gate(&gate, !l_color_false_bit, r_color_false_bit),
        );
        let o_true_true = encoded_wire_o.false_value.mux(
            &encoded_wire_o.true_value,
            &Self::lookup_gate(&gate, !l_color_false_bit, !r_color_false_bit),
        );

        let wire_o_index_bits = B::from(gate.wire_o);
        let input_false_false = l_color_false
            .concat(&r_color_false)
            .concat(&wire_o_index_bits)
            .concat(&B::from_const_bits(&[false, false]));
        let ct_false_false = self.hash_fn.compute(&input_false_false)? ^ o_false_false;
        let input_false_true = l_color_false
            .concat(&r_color_true)
            .concat(&wire_o_index_bits)
            .concat(&B::from_const_bits(&[false, true]));
        let ct_false_true = self.hash_fn.compute(&input_false_true)? ^ o_false_true;
        let input_true_false = l_color_true
            .concat(&r_color_false)
            .concat(&wire_o_index_bits)
            .concat(&B::from_const_bits(&[true, false]));
        let ct_true_false = self.hash_fn.compute(&input_true_false)? ^ o_true_false;
        let input_true_true = l_color_true
            .concat(&r_color_true)
            .concat(&wire_o_index_bits)
            .concat(&B::from_const_bits(&[true, true]));
        let ct_true_true = self.hash_fn.compute(&input_true_true)? ^ o_true_true;
        Ok(EncodedGate {
            gate,
            cts: [ct_false_false, ct_false_true, ct_true_false, ct_true_true],
        })

        /*match gate.gate_type {
            GateType::Not | GateType::Xor => return Ok(None),
            _ => {}
        };
        let wire_l = self.encode_wire(gate.wire_l)?;
        let wire_r = self.encode_wire(gate.wire_r)?;
        let wire_g_o = self.encode_wire(gate.wire_o)?;
        let p_l = wire_l.value.get_lsb();
        let p_r = wire_r.value.get_lsb();
        let (select_l, select_r, select_o) = match gate.gate_type {
            GateType::And => (<B::Element as Bit>::from_const_bit(false),<B::Element as Bit>::from_const_bit(false),<B::Element as Bit>::from_const_bit(false)),
            _ => (<B::Element as Bit>::from_const_bit(true),<B::Element as Bit>::from_const_bit(true),<B::Element as Bit>::from_const_bit(true))
        };
        let false_bits = B::from_const_bits(&false_const_bits);

        //Generate a ciphertext for generator-known input.
        let index_bits = B::from(gate.wire_o);
        let hash_l_0 = self.hash_fn.compute(&wire_l.value.concat(&index_bits))?;
        let hash_l_1 = self.hash_fn.compute(&(wire_l.value ^ self.master_r).concat(&index_bits))?;
        let gen_out_bit_0 = (<B::Element as Bit>::from_const_bit(false) ^ select_l) & (p_r ^ select_r) ^ select_o;
        let gen_out_bit_1 = (<B::Element as Bit>::from_const_bit(true) ^ select_l) & (p_r ^ select_r) ^ select_o;
        let gen_r_0 = false_bits.mux(&self.master_r,&gen_out_bit_0);
        let gen_r_1 = false_bits.mux(&self.master_r,&gen_out_bit_1);
        let gen_ct0 = hash_l_0 ^ gen_r_0 ^ wire_o;
        let gen_ct1 = hash_l_1 ^ gen_r_1 ^ wire_o;

        let hash_r_0 = self.hash_fn.compute(&wire_r.value.concat(&index_bits))?;
        let hash_r_1 = self.hash_fn.compute(&(wire_r.value ^ self.master_r).concat(&index_bits))?;
        let mut false_const_bits = Vec::new();
        for _ in 0..<H as HashFn<B>>::OUT_SIZE {
            false_const_bits.push(false);
        }
        let false_bits = B::from_const_bits(&false_const_bits);

        let gen_r1 = false_bits.mux(&self.master_r,&(p_r^select_r));
        let generator_ct = hash_l_0 ^ hash_l_1 ^ gen_r1;
        let gen_r2 =
        let gen_wire = */
    }

    fn lookup_gate(gate: &Gate, l: B::Element, r: B::Element) -> B::Element {
        match gate.gate_type {
            GateType::Not => !l,
            GateType::And => l & r,
            GateType::Xor => l ^ r,
            GateType::Or => l | r,
        }
    }
}
