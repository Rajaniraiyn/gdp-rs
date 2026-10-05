// error-pattern: subject name conflicts with a generated accessor
#[gp::proof(subjects(subject_0))]
pub struct P<'a>;
fn main() {}
