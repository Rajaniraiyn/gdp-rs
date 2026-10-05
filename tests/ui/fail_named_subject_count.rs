// error-pattern: expected 2 subject names
#[gp::proof(subjects(actor))]
pub struct P<'a, 'b>;
fn main() {}
