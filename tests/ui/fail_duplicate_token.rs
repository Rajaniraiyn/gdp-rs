// error-pattern: use of moved value
mod support;fn main(){gp::name!(u=1_u32,p=2_u32);let proof=support::policy::admin(&u,&p);let cap=proof.bind(u,p);support::policy::consume(cap);support::policy::consume(cap);}
