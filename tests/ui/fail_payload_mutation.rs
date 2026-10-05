// error-pattern: field `row`
mod support;fn main(){gp::name!(a=1_u32);let mut p=support::policy::payload(&a);p.row=9;}
