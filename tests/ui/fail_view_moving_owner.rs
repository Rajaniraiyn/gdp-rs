// error-pattern: cannot move out
mod support;
fn main() {
    gp::name!(u=1_u32,p=2_u32);
    let proof=support::policy::admin(&u,&p);
    let cap=proof.bind(u,p);
    let view=cap.as_view();
    support::policy::consume(cap);
    let _=view.proof();
}
