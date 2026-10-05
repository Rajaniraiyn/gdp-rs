mod policy {
    #[gp::proof]
    pub struct Snapshot<'a, T: Clone = u32, const N: usize = 2>
    where T: PartialEq {
        values: [T; N],
    }
    pub fn check<'a, T: Clone + PartialEq, const N: usize>(a: &gp::Named<'a, [T; N]>) -> Snapshot<'a, T, N> {
        Snapshot::issue(a, a.value().clone())
    }
    impl<'a, T: Clone + PartialEq, const N: usize> Snapshot<'a, T, N> {
        pub fn values(&self) -> &[T; N] { &self.values }
    }
}
fn main() {
    gp::name!(a = [1_u32, 2]);
    let proof = policy::check(&a);
    assert_eq!(proof.values(), &[1, 2]);
    let view = proof.view(&a);
    assert_eq!(view.subject_0().value(), view.proof().values());
    let cap = proof.bind(a);
    assert_eq!(cap.subject_0().value(), cap.proof().values());
}
