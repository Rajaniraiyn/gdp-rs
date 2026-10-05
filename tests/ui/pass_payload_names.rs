mod policy {
    #[gp::proof(subjects(resource))]
    pub struct P<'a> {
        subject_0: u32,
        __gdp_subject_0: u32,
        __gdp_subject_0_: u32,
    }
    pub fn check<'a>(value: &gp::Named<'a, u32>) -> P<'a> {
        P::issue(value, *value.value(), 2, 3)
    }
    impl P<'_> {
        pub fn total(&self) -> u32 { self.subject_0 + self.__gdp_subject_0 + self.__gdp_subject_0_ }
    }
}
fn main() {
    gp::name!(value = 1_u32);
    let proof = policy::check(&value);
    let capability = proof.bind(value);
    assert_eq!(*capability.resource().value(), 1);
    assert_eq!(capability.proof().total(), 6);
}
