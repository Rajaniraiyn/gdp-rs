// error-pattern: proof payload fields must be private
mod p{#[gp::proof]pub struct P<'a>{pub row:u32}}fn main(){}
