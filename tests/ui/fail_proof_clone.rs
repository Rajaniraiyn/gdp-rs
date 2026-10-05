// error-pattern: no method named `clone`
mod support;fn main(){gp::name!(u=1_u32,p=2_u32);let p=support::policy::admin(&u,&p);let _=p.clone();}
