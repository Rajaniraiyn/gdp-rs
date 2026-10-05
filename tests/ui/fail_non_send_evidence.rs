// error-pattern: cannot be sent between threads safely
mod p {
    #[gp::proof] pub struct Payload<'a> { data: std::rc::Rc<u32> }
    pub fn check<'a>(a: &gp::Named<'a, u32>) -> Payload<'a> { Payload::issue(a, std::rc::Rc::new(*a.value())) }
}
fn send<T: Send>(_: T) {}
fn main() { gp::name!(a=1_u32); send(p::check(&a)); }
