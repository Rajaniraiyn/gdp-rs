/// Conjunction containing actual evidence for both components.
///
/// Components can be proofs, borrowed proofs, or data carrying evidence.
/// Construction adds no trust; it only packages the supplied components.
#[must_use]
#[derive(Debug)]
pub struct And<A, B> {
    left: A,
    right: B,
}

impl<A, B> And<A, B> {
    /// Combine two existing components without cloning either.
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }

    /// Borrow the left component.
    pub fn left(&self) -> &A {
        &self.left
    }

    /// Borrow the right component.
    pub fn right(&self) -> &B {
        &self.right
    }

    /// Consume the conjunction and recover both components.
    pub fn into_parts(self) -> (A, B) {
        (self.left, self.right)
    }
}

/// Disjunction containing actual evidence for the selected alternative.
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

    /// Borrow the selected evidence without duplicating it.
    pub fn as_ref(&self) -> Either<&A, &B> {
        match self {
            Self::Left(a) => Either::Left(a),
            Self::Right(b) => Either::Right(b),
        }
    }
}
