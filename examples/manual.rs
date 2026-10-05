//! GDP without procedural macros or an allocator in the library.
use core::marker::PhantomData;
use ghostproof::{Named, name};

mod positive {
    use super::*;
    pub struct Positive<'id> {
        _brand: PhantomData<fn(&'id ()) -> &'id ()>,
    }
    pub fn check<'id>(value: &Named<'id, i32>) -> Option<Positive<'id>> {
        (*value.value() > 0).then_some(Positive {
            _brand: PhantomData,
        })
    }
}

fn read_positive<'id>(value: &Named<'id, i32>, _: &positive::Positive<'id>) -> i32 {
    *value.value()
}

fn main() {
    name!(value = 4);
    let proof = positive::check(&value).expect("positive");
    assert_eq!(read_positive(&value, &proof), 4);
}
