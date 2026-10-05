//! A ternary relationship and evidence alternatives.
use ghostproof::{Either, Named, name};

mod arithmetic {
    use super::*;
    #[ghostproof::proof]
    pub struct Sum<'a, 'b, 'total>;
    #[ghostproof::proof]
    pub struct Zero<'a>;

    pub fn sum<'a, 'b, 't>(
        a: &Named<'a, u32>,
        b: &Named<'b, u32>,
        total: &Named<'t, u32>,
    ) -> Option<Sum<'a, 'b, 't>> {
        (a.value().checked_add(*b.value()) == Some(*total.value())).then(|| Sum::issue(a, b, total))
    }
    pub fn zero<'a>(a: &Named<'a, u32>) -> Option<Zero<'a>> {
        (*a.value() == 0).then(|| Zero::issue(a))
    }
}

fn main() {
    name!(a = 0_u32, b = 4_u32, total = 4_u32);
    let relation = arithmetic::sum(&a, &b, &total).unwrap();
    let alternatives: Either<arithmetic::Sum<'_, '_, '_>, arithmetic::Zero<'_>> =
        Either::Left(relation);
    assert_eq!(alternatives.fold(|_| "sum", |_| "zero"), "sum");
    let zero = arithmetic::zero(&a).unwrap().bind(a);
    assert_eq!(*zero.subject_0().value(), 0);
}
