mod p {
 #[gp::proof] pub struct Relation<'a,'b,'c,'d>;
 pub fn check<'a,'b,'c,'d>(a:&gp::Named<'a,u8>,b:&gp::Named<'b,u8>,c:&gp::Named<'c,u8>,d:&gp::Named<'d,u8>)->Relation<'a,'b,'c,'d> { Relation::issue(a,b,c,d) }
}
fn main(){ gp::name!(a=1_u8,b=2_u8,c=3_u8,d=4_u8);let cap=p::check(&a,&b,&c,&d).bind(a,b,c,d);assert_eq!(*cap.subject_3().value(),4); }
