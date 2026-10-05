mod policy {
    #[gp::proof]
    pub struct Typed<'id, T: ?Sized>;
    pub fn check<'id>(a: &gp::Named<'id, u32>) -> Typed<'id, str> { Typed::issue(a) }
}
fn main() {
    gp::name!(a=1_u32);
    let cap=policy::check(&a).bind(a);
    assert_eq!(*cap.subject_0().value(), 1);
}
