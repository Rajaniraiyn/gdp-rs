// error-pattern: lifetime may not live long enough
mod policy { #[gp::proof] pub struct Typed<'id, T>; }
fn widen<'id, 'a, 'b>(p: policy::Typed<'id, &'a u32>) -> policy::Typed<'id, &'b u32>
where 'a: 'b { p }
fn main() {}
