//! Ghosts of Departed Proofs for Rust.
//!
//! Name values with fresh invariant brands, check facts in a trusted module,
//! and require evidence about those exact names in dependent operations.
//! The library is `no_std`, does not allocate, and does not select an executor.
//!
//! ```
//! use ghostproof::{name, Named};
//! name!(item = 42);
//! assert_eq!(*item.value(), 42);
//! ```
//!
//! Applications own the truth of their checks. Naming does not freeze interior
//! mutability or external state. Use immutable snapshots for validation and
//! appropriate database enforcement for facts that can change between operations.

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
