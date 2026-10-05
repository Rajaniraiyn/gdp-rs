//! Validate an immutable snapshot and carry useful evidence about it.
use ghostproof::{Named, name};

mod sorted {
    use super::*;
    #[ghostproof::proof]
    pub struct Sorted<'sequence> {
        len: usize,
    }

    impl Sorted<'_> {
        pub fn len(&self) -> usize {
            self.len
        }
    }

    pub fn check<'s>(sequence: &Named<'s, Vec<i32>>) -> Result<Sorted<'s>, &'static str> {
        let values = sequence.value();
        if values.windows(2).all(|w| w[0] <= w[1]) {
            Ok(Sorted::issue(sequence, values.len()))
        } else {
            Err("sequence is not sorted")
        }
    }

    impl<'s> SortedCapability<'s, Vec<i32>> {
        pub fn find(&self, needle: i32) -> Result<usize, usize> {
            self.subject_0().value().binary_search(&needle)
        }
        // A transition discards previous evidence and checks the new snapshot.
        pub fn append(self, value: i32) -> Vec<i32> {
            let ((sequence,), _old_evidence) = self.into_parts();
            let mut values = sequence.into_inner();
            values.push(value);
            values
        }
    }
}

fn main() {
    name!(sequence = vec![1, 3, 5]);
    let proof = sorted::check(&sequence).unwrap();
    assert_eq!(proof.len(), 3);
    let checked = proof.bind(sequence);
    assert_eq!(checked.find(3), Ok(1));
    name!(next = checked.append(7));
    let next = sorted::check(&next).unwrap().bind(next);
    assert_eq!(next.find(7), Ok(3));
    name!(bad = vec![5, 1]);
    assert!(sorted::check(&bad).is_err());
}
