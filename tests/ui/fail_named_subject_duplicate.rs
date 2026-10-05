// error-pattern: subject names must be distinct
#[gp::proof(subjects(actor, r#actor))]
pub struct P<'a, 'b>;
fn main() {}
