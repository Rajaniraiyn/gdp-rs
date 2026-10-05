// error-pattern: use of moved value
fn main(){gp::make_guard!(guard);let _a=gp::Named::new(1,guard);let _b=gp::Named::new(2,guard);}
