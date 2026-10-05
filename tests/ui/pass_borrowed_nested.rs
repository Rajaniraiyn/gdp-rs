fn main(){let text=String::from("hello"); gp::name!(outer=text.as_str()); for _ in 0..2 { gp::name!(inner=outer.value().len());assert_eq!(*inner.value(),5); } assert_eq!(*outer.value(),"hello");}
