// error-pattern: mismatched types
mod support;fn main(){gp::name!(u=1_u32,p=2_u32);let proof=support::policy::plan(&p);support::policy::operation(&u,&p,&proof);}
