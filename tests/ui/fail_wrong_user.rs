// error-pattern: E0597|E0716
mod support;fn main(){gp::name!(u=1_u32,v=1_u32,p=2_u32);let proof=support::policy::admin(&u,&p);support::policy::operation(&v,&p,&proof);}
