// error-pattern: cannot be sent between threads safely
fn assert_send<T:Send>(_:T){}fn main(){gp::name!(a=std::rc::Rc::new(1));assert_send(a);}
