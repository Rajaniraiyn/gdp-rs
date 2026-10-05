// error-pattern: E0451
mod support;fn main(){gp::name!(u=1_u32,p=2_u32);let proof=support::policy::admin(&u,&p);let _=support::policy::AdminCapability{subject_0:u,subject_1:p,proof};}
