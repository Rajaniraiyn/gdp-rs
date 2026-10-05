// error-pattern: E0597|E0716
mod support;fn spawn<T:Send+'static>(_:T){}fn main(){gp::name!(u=1_u32,p=2_u32);let proof=support::policy::admin(&u,&p);spawn(async move{support::policy::operation(&u,&p,&proof)});}
