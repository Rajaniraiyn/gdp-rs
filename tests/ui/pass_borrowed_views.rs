mod support;
fn main() {
    gp::name!(u=1_u32,p=2_u32);
    let admin = support::policy::admin(&u,&p);
    let plan = support::policy::plan(&p);
    let view = admin.view(&u,&p);
    fn require_send<T: Send>(_: &T) {}
    require_send(&view);
    let other = plan.view(&p);
    support::policy::operation(view.subject_0(),view.subject_1(),view.proof());
    assert_eq!(other.subject_0().value(),view.subject_1().value());
    let cap = admin.bind(u,p);
    let borrowed = cap.as_view();
    assert_eq!(*borrowed.subject_1().value(),2);
    support::policy::consume(cap);
}
