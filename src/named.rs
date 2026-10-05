use generativity::{Guard, Id};

/// An owned value identified by a fresh, invariant lifetime brand.
///
/// Payload access is shared; extraction consumes the wrapper. Interior mutability
/// in `T` remains available through shared references.
#[must_use = "use the named value in a check or dependent operation"]
#[derive(Debug)]
pub struct Named<'id, T> {
    value: T,
    _brand: Id<'id>,
}

impl<'id, T> Named<'id, T> {
    /// Consume a fresh guard to name exactly one value.
    pub fn new(value: T, guard: Guard<'id>) -> Self {
        Self {
            value,
            _brand: guard.into(),
        }
    }

    /// Borrow the underlying value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consume the wrapper and recover its underlying value.
    ///
    /// The returned value has its original unbranded type.
    pub fn into_inner(self) -> T {
        self.value
    }
}
