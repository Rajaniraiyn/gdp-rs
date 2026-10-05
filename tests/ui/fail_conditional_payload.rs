// error-pattern: condition the whole proof declaration
#[gp::proof]
pub struct Conditional<'a> { #[cfg(any())] payload: u32 }
fn main() {}
