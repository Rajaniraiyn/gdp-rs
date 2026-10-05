//! Runtime behavior, ownership, payload, and async contracts.
use ghostproof::{And, Either, Named, make_guard, name};

#[test]
fn names_support_borrows_loops_shadowing_and_raw_identifiers() {
    let source = String::from("snapshot");
    name!(borrowed = source.as_str());
    assert_eq!(*borrowed.value(), "snapshot");
    let value = 9;
    name!(value = value, r#type = 12);
    assert_eq!(*value.value(), 9);
    assert_eq!(*r#type.value(), 12);
    for i in 0..4 {
        make_guard!(guard);
        let item = Named::new(i, guard);
        assert_eq!(item.into_inner(), i);
    }
    let raw = {
        name!(inner = 4);
        inner.into_inner()
    };
    assert_eq!(raw, 4);
}

#[test]
fn composition_moves_or_borrows_existing_components() {
    let left = String::from("a");
    let right = String::from("b");
    let borrowed = And::new(&left, &right);
    assert_eq!(borrowed.left().as_str(), "a");
    assert_eq!(borrowed.right().as_str(), "b");
    let owned = And::new(left, right);
    let (left, right) = owned.into_parts();
    let choice: Either<String, String> = Either::Left(left);
    assert!(matches!(choice.as_ref(), Either::Left(_)));
    assert_eq!(choice.fold(|a| a, |b| b), "a");
    let choice: Either<String, String> = Either::Right(right);
    assert_eq!(choice.fold(|a| a, |b| b), "b");
}

#[cfg(feature = "macros")]
mod proofs {
    use super::*;
    use std::future::Future;
    use std::{
        cell::Cell,
        rc::Rc,
        task::{Context, Poll},
    };

    mod policy {
        use super::*;
        #[ghostproof::proof]
        pub struct Equal<'a, 'b>;
        #[ghostproof::proof]
        pub struct Positive<'a> {
            value: u32,
        }
        #[ghostproof::proof]
        pub struct Ticket<'a> {
            tracker: DropTracker,
        }
        #[ghostproof::proof]
        pub struct r#Checked<'a>;

        pub fn equal<'a, 'b>(a: &Named<'a, u32>, b: &Named<'b, u32>) -> Option<Equal<'a, 'b>> {
            (a.value() == b.value()).then(|| Equal::issue(a, b))
        }
        pub fn positive<'a>(a: &Named<'a, u32>) -> Option<Positive<'a>> {
            (*a.value() > 0).then(|| Positive::issue(a, *a.value()))
        }
        impl Positive<'_> {
            pub fn value(&self) -> u32 {
                self.value
            }
        }
        // Payload transformation keeps the proposition and brand unchanged.
        impl<'a> Positive<'a> {
            pub fn refresh_payload(self, a: &Named<'a, u32>) -> Self {
                Self::issue(a, self.value)
            }
        }
        pub fn ticket<'a>(a: &Named<'a, u32>, tracker: DropTracker) -> Ticket<'a> {
            Ticket::issue(a, tracker)
        }
        impl<'a> TicketCapability<'a, u32> {
            pub fn consume(self) -> u32 {
                *self.subject_0().value()
            }
        }
        impl Drop for Ticket<'_> {
            fn drop(&mut self) {
                let _ = &self.tracker;
            }
        }
        pub fn raw<'a>(a: &Named<'a, u32>) -> r#Checked<'a> {
            r#Checked::issue(a)
        }
    }

    pub struct DropTracker(Rc<Cell<usize>>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn checks_and_capabilities_keep_payloads_and_subjects() {
        name!(a = 4_u32, b = 4_u32, bad = 0_u32);
        let equal = policy::equal(&a, &b).unwrap();
        let positive = policy::positive(&a).unwrap().refresh_payload(&a);
        assert_eq!(positive.value(), 4);
        assert!(policy::positive(&bad).is_none());
        assert!(policy::equal(&a, &bad).is_none());
        let pair = equal.bind(a, b);
        assert_eq!(pair.subject_0().value(), pair.subject_1().value());
        let ((a, b), equal) = pair.into_parts();
        let pair = equal.bind(a, b);
        assert_eq!(*pair.subject_1().value(), 4);
        name!(raw_subject = 1_u32);
        let capability = policy::raw(&raw_subject).bind(raw_subject);
        assert_eq!(*capability.subject_0().value(), 1);
    }

    #[test]
    fn borrowed_views_share_subjects_without_consuming_owned_permissions() {
        name!(a = 4_u32, b = 4_u32);
        let equal = policy::equal(&a, &b).unwrap();
        let positive = policy::positive(&a).unwrap();
        let first = equal.view(&a, &b);
        let second = positive.view(&a);
        assert!(std::ptr::eq(first.subject_0(), second.subject_0()));
        assert!(std::ptr::eq(first.proof(), &equal));
        assert_eq!(second.proof().value(), 4);
        let owned = equal.bind(a, b);
        let view = owned.as_view();
        assert!(std::ptr::eq(view.subject_0(), owned.subject_0()));
        let ((a, _b), _proof) = owned.into_parts();
        let drops = Rc::new(Cell::new(0));
        let ticket = policy::ticket(&a, DropTracker(drops.clone()));
        {
            let view = ticket.view(&a);
            assert_eq!(*view.subject_0().value(), 4);
        }
        assert_eq!(drops.get(), 0);
        assert_eq!(ticket.bind(a).consume(), 4);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn cancellation_and_unwinding_release_owned_evidence() {
        let drops = Rc::new(Cell::new(0));
        let tracker = DropTracker(drops.clone());
        let mut future = Box::pin(async move {
            name!(subject = 1_u32);
            let capability = policy::ticket(&subject, tracker).bind(subject);
            futures::pending!();
            capability.consume()
        });
        let mut context = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
        drop(future);
        assert_eq!(drops.get(), 1);

        let tracker = DropTracker(drops.clone());
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            name!(subject = 2_u32);
            let _capability = policy::ticket(&subject, tracker).bind(subject);
            panic!("operation failed");
        }));
        assert!(outcome.is_err());
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn scoped_evidence_survives_await_and_borrowing_concurrency() {
        fn assert_send<T: Send>(_: &T) {}
        fn assert_sync<T: Sync>(_: &T) {}
        let future = async {
            name!(a = 3_u32, b = 3_u32);
            let proof = policy::equal(&a, &b).unwrap();
            assert_send(&proof);
            assert_sync(&proof);
            let capability = proof.bind(a, b);
            let read = async {
                futures::future::ready(()).await;
                *capability.subject_0().value()
            };
            let other = async { *capability.subject_1().value() };
            let (a, b) = futures::join!(read, other);
            assert_eq!(a, b);
        };
        assert_send(&future);
        futures::executor::block_on(future);
    }

    #[test]
    fn phantom_proofs_and_brands_have_zero_size() {
        name!(a = 1_u32, b = 1_u32);
        let proof = policy::equal(&a, &b).unwrap();
        assert_eq!(std::mem::size_of_val(&proof), 0);
        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of::<u32>());
    }
}
