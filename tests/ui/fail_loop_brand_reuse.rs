// error-pattern: E0597|E0716
fn main(){gp::name!(outer=0);for i in 0..2 {gp::name!(inner=i);let _both=[outer.value(),inner.value()];fn same<'a>(_:&gp::Named<'a,i32>,_:&gp::Named<'a,i32>){}same(&outer,&inner);}}
