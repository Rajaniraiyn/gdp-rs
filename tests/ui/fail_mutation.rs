// error-pattern: cannot assign
fn main(){gp::name!(a=1_u32);*a.value()=2;}
