mod policy {
    #[gp::proof(subjects(actor, r#type))]
    pub struct Equal<'a, 'b, T: PartialEq = u32> { sample: T }
    pub fn check<'a, 'b>(a: &gp::Named<'a, u32>, b: &gp::Named<'b, u32>) -> Option<Equal<'a, 'b>> {
        (a.value() == b.value()).then(|| Equal::issue(a, b, *a.value()))
    }
    impl Equal<'_, '_> { pub fn sample(&self) -> u32 { self.sample } }
}
fn main() {
    gp::with_names!(actor = 7_u32, resource = 7_u32; {
        let proof = policy::check(&actor, &resource).unwrap();
        let view = proof.view(&actor, &resource);
        assert_eq!(view.actor().value(), view.r#type().value());
        let capability = proof.bind(actor, resource);
        assert!(core::ptr::eq(capability.actor(), capability.subject_0()));
        assert_eq!(*capability.as_view().r#type().value(), capability.proof().sample());
    });
}
