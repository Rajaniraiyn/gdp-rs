// error-pattern: subject name conflicts with a generated accessor
#[gp::proof(subjects(proof))]
pub struct P<'a>;
fn main() {}
