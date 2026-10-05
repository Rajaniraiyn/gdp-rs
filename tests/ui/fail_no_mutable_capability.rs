// error-pattern: no method named `subject_0_mut`
mod support;fn main(){gp::name!(a=1_u32);let mut cap=support::policy::payload(&a).bind(a);let _=cap.subject_0_mut();}
