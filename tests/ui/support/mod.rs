#![allow(dead_code)]
use gp::Named;

pub mod policy {
    use super::*;
    #[gp::proof]
    pub struct Admin<'u, 'p>;
    #[gp::proof]
    pub struct Plan<'p>;
    #[gp::proof]
    pub struct Sum<'a, 'b, 'c>;
    #[gp::proof]
    pub struct Payload<'a> { row: u32 }

    pub fn admin<'u, 'p>(u: &Named<'u, u32>, p: &Named<'p, u32>) -> Admin<'u, 'p> { Admin::issue(u, p) }
    pub fn plan<'p>(p: &Named<'p, u32>) -> Plan<'p> { Plan::issue(p) }
    pub fn sum<'a, 'b, 'c>(a: &Named<'a, u32>, b: &Named<'b, u32>, c: &Named<'c, u32>) -> Sum<'a, 'b, 'c> { Sum::issue(a, b, c) }
    pub fn payload<'a>(a: &Named<'a, u32>) -> Payload<'a> { Payload::issue(a, *a.value()) }
    impl Payload<'_> { pub fn row(&self) -> u32 { self.row } }
    pub fn operation<'u, 'p>(u: &Named<'u, u32>, p: &Named<'p, u32>, _: &Admin<'u, 'p>) { let _ = (u, p); }
    pub fn total<'a, 'b, 'c>(a: &Named<'a, u32>, b: &Named<'b, u32>, c: &Named<'c, u32>, _: &Sum<'a, 'b, 'c>) { let _ = (a, b, c); }
    pub fn consume<'u, 'p>(cap: AdminCapability<'u, 'p, u32, u32>) { let _ = cap; }
}
