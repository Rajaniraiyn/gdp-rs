// error-pattern: E0597|E0716
mod support;fn main(){gp::name!(a=1_u32,b=2_u32,c=3_u32,other=3_u32);let proof=support::policy::sum(&a,&b,&c);support::policy::total(&a,&other,&c,&proof);}
