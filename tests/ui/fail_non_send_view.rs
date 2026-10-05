// error-pattern: cannot be shared between threads safely
mod policy {
    #[gp::proof] pub struct Fact<'a>;
    pub fn check<'a>(a: &gp::Named<'a, std::cell::Cell<u32>>) -> Fact<'a> { Fact::issue(a) }
}
fn require_send<T: Send>(_: T) {}
fn main() {
    gp::name!(a=std::cell::Cell::new(1_u32));
    let proof=policy::check(&a);
    require_send(proof.view(&a));
}
