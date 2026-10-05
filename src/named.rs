use generativity::{Guard, Id};

/// An owned value identified by a fresh, invariant lifetime brand.
///
/// The wrapper is neither `Copy` nor `Clone`. It permits no replacement or
/// mutable access. Shared access does not prevent interior mutation inside `T`.
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

    /// Read the underlying value without replacing it.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consume the wrapper and recover its underlying value.
    ///
    /// Existing evidence is not automatically attached to the recovered value.
    pub fn into_inner(self) -> T {
        self.value
    }
}
