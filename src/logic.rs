/// Conjunction containing both evidence components.
///
/// Components can be proofs, borrowed proofs, or data carrying evidence.
/// Stores the supplied components.
#[must_use]
#[derive(Debug)]
pub struct And<A, B> {
    left: A,
    right: B,
}

impl<A, B> And<A, B> {
    /// Move two components into a conjunction.
    pub const fn new(left: A, right: B) -> Self {
        Self { left, right }
    }

    /// Borrow the left component.
    pub const fn left(&self) -> &A {
        &self.left
    }

    /// Borrow the right component.
    pub const fn right(&self) -> &B {
        &self.right
    }

    /// Consume the conjunction and recover both components.
    pub fn into_parts(self) -> (A, B) {
        (self.left, self.right)
    }

    /// Borrow both components.
    pub const fn as_ref(&self) -> And<&A, &B> {
        And::new(&self.left, &self.right)
    }
}

/// Evidence for the selected alternative.
#[must_use]
#[derive(Debug)]
pub enum Either<A, B> {
    /// Evidence for the first alternative.
    Left(A),
    /// Evidence for the second alternative.
    Right(B),
}

impl<A, B> Either<A, B> {
    /// Handle either branch, consuming its evidence.
    pub fn fold<R>(self, left: impl FnOnce(A) -> R, right: impl FnOnce(B) -> R) -> R {
        match self {
            Self::Left(a) => left(a),
            Self::Right(b) => right(b),
        }
    }

    /// Borrow the selected evidence.
    pub const fn as_ref(&self) -> Either<&A, &B> {
        match self {
            Self::Left(a) => Either::Left(a),
            Self::Right(b) => Either::Right(b),
        }
    }
}
