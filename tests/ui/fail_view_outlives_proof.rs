// error-pattern: does not live long enough
mod support;
fn main() {
    gp::name!(u=1_u32,p=2_u32);
    let view = { let proof=support::policy::admin(&u,&p); proof.view(&u,&p) };
    let _ = view.subject_1();
}
