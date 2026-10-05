//! Scoped naming, composition, and const API contracts.
use ghostproof::{And, Either, all, with_names};

#[test]
fn scoped_names_evaluate_once_and_preserve_payload_borrows() {
    let mut order = Vec::new();
    let source = String::from("snapshot");
    let result = with_names!(
        first = { order.push(1); 3 },
        second = { order.push(2); 4 },
        text = source.as_str(),;
        { (*first.value() + *second.value(), *text.value()) }
    );
    assert_eq!(order, [1, 2]);
    assert_eq!(result, (7, "snapshot"));
}

#[test]
fn scoped_names_support_shadowing_raw_identifiers_and_const_expressions() {
    let value = 5;
    let result = with_names!(value = value, r#type = const { 2 + 3 }; {
        *value.value() + *r#type.value()
    });
    assert_eq!(result, 10);
    assert_eq!(value, 5);
}

#[test]
fn scoped_names_survive_await_inside_the_body() {
    let result = futures::executor::block_on(async {
        with_names!(item = String::from("checked"); {
            async {}.await;
            item.into_inner()
        })
    });
    assert_eq!(result, "checked");
}

#[test]
fn conjunction_macro_evaluates_in_order_and_moves_owned_components() {
    let mut order = Vec::new();
    let facts = all!(
        {
            order.push(1);
            String::from("a")
        },
        {
            order.push(2);
            String::from("b")
        },
        {
            order.push(3);
            String::from("c")
        },
    );
    assert_eq!(order, [1, 2, 3]);
    let borrowed = facts.as_ref();
    assert!(std::ptr::eq(*borrowed.left(), facts.left()));
    let (a, rest) = facts.into_parts();
    let (b, c) = rest.into_parts();
    assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("a", "b", "c"));
}

#[test]
fn conjunction_macro_borrows_existing_components() {
    let a = String::from("a");
    let b = String::from("b");
    let c = String::from("c");
    let d = String::from("d");
    let facts = all!(&a, &b, &c, &d);
    assert!(std::ptr::eq(*facts.left(), &a));
    assert!(std::ptr::eq(*facts.right().right().right(), &d));
    assert_eq!(a, "a");
}

#[test]
fn composition_supports_const_construction_and_borrowing() {
    const PAIR: And<u32, bool> = And::new(7, true);
    const BORROWED: And<&u32, &bool> = PAIR.as_ref();
    const CHOICE: Either<u32, bool> = Either::Left(9);
    const VIEW: Either<&u32, &bool> = CHOICE.as_ref();
    ghostproof::const_assert!(*PAIR.left() == 7);
    ghostproof::const_assert!(*PAIR.right());
    assert_eq!(**BORROWED.left(), 7);
    assert!(matches!(VIEW, Either::Left(&9)));
}
