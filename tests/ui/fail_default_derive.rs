// error-pattern: proofs cannot derive
mod p{#[gp::proof]#[derive(Default)]pub struct P<'a>;}fn main(){}
