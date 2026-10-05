//! Ghosts of Departed Proofs for Rust.
//!
//! Name values with fresh invariant brands, check facts in a trusted module,
//! and require evidence about those exact names in dependent operations.
//! The core is allocation-free and `no_std`.
//!
//! ```
//! use ghostproof::{name, Named};
//! name!(item = 42);
//! assert_eq!(*item.value(), 42);
//! ```
//!
//! Applications implement checker semantics. Validate immutable snapshots and
//! enforce changing external facts with suitable atomic storage operations.

#![no_std]
#![forbid(unsafe_code)]

extern crate self as ghostproof;

mod logic;
mod named;

pub use generativity::{Guard, make_guard};
pub use logic::{And, Either};
pub use named::Named;

#[cfg(feature = "macros")]
pub use ghostproof_macros::proof;

/// Give each declared value a fresh name in the current scope.
///
/// Values may be borrowed. The brand scope is independent of the payload's
/// borrow lifetime. A named wrapper cannot leave its naming scope.
///
/// ```
/// ghostproof::name!(first = 7, second = 7);
/// assert_eq!(first.value(), second.value());
/// // Their brands are still distinct even though their values compare equal.
/// ```
#[macro_export]
macro_rules! name {
    ($($binding:ident = $value:expr),+ $(,)?) => {
        $(
            let $binding = $value;
            $crate::make_guard!(__gdp_guard);
            let $binding = $crate::Named::new($binding, __gdp_guard);
        )+
    };
}

/// Name values within a block and return an ordinary result.
///
/// Names and their evidence cannot leave the block. Use this inside an async
/// function to keep checks and `.await` operations in the same naming scope.
///
/// ```
/// let value = ghostproof::with_names!(a = 7, b = 9; {
///     a.value() + b.value()
/// });
/// assert_eq!(value, 16);
/// ```
#[macro_export]
macro_rules! with_names {
    ($($binding:ident = $value:expr),+ $(,)?; $body:block) => {{
        $crate::name!($($binding = $value),+);
        $body
    }};
}

/// Combine two or more existing facts into a right-nested conjunction.
///
/// `all!(a, b, c)` has type `And<A, And<B, C>>`. Expressions are evaluated once,
/// from left to right. Pass references to borrow reusable facts.
///
/// ```
/// let facts = ghostproof::all!("first", 2, true);
/// assert_eq!(*facts.left(), "first");
/// assert_eq!(*facts.right().left(), 2);
/// assert!(*facts.right().right());
/// ```
#[macro_export]
macro_rules! all {
    ($first:expr, $second:expr $(,)?) => {
        $crate::And::new($first, $second)
    };
    ($first:expr, $second:expr, $($rest:expr),+ $(,)?) => {
        $crate::And::new($first, $crate::all!($second, $($rest),+))
    };
}

/// Assert a constant condition during compilation.
///
/// Use at module or block scope with a const expression and an optional string
/// literal message. A false condition produces a constant-evaluation error.
///
/// ```
/// const BATCH: usize = 64;
/// ghostproof::const_assert!(BATCH > 0, "batch size must be positive");
/// ```
#[macro_export]
macro_rules! const_assert {
    ($condition:expr $(,)?) => {
        const _: () = {
            ::core::assert!($condition);
        };
    };
    ($condition:expr, $message:literal $(,)?) => {
        const _: () = {
            ::core::assert!($condition, $message);
        };
    };
}
