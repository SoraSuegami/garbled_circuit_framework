use crate::Bits;
use std::error::Error as TError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashFnError {
    #[error("{0}")]
    InnerHashFnError(String),
}

#[derive(Error, Debug)]
pub enum PRFError {
    #[error("{0}")]
    InnerPRFError(String),
}

pub trait HashFn<B: Bits> {
    const OUT_SIZE: usize;
    type Error: TError;
    fn compute(&self, input: &B) -> Result<B, HashFnError>;
}

pub trait PRF<B: Bits> {
    const KEY_SIZE: usize;
    fn get_key(&self) -> Result<B, PRFError>;
    fn compute(&self, seed: &B) -> Result<B, PRFError>;
    fn modify_key(&mut self, key: B);
}
