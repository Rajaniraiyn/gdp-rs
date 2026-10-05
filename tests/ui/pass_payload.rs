mod support;fn main(){gp::name!(a=5_u32);let proof=support::policy::payload(&a);assert_eq!(proof.row(),5);let cap=proof.bind(a);assert_eq!(cap.proof().row(),5);}
