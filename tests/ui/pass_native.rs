mod support;
fn main() {
 gp::name!(u=1_u32, p=2_u32);
 let proof=support::policy::admin(&u,&p);
 support::policy::operation(&u,&p,&proof);
 let cap=proof.bind(u,p);
 assert_eq!(*cap.subject_1().value(),2);
 support::policy::consume(cap);
}
