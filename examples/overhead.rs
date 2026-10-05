//! A small runtime comparison with the same check in both paths.
use ghostproof::{Named, name};
use std::{hint::black_box, time::Instant};

mod positive {
    use super::*;
    #[ghostproof::proof]
    pub struct Positive<'id>;
    pub fn check<'id>(value: &Named<'id, u64>) -> Option<Positive<'id>> {
        (*value.value() > 0).then(|| Positive::issue(value))
    }
}

fn plain(iterations: u64) -> u64 {
    let mut total = 0_u64;
    for i in 0..iterations {
        let value = black_box(i);
        if value > 0 {
            total = total.wrapping_add(black_box(value));
        }
    }
    black_box(total)
}

fn proved(iterations: u64) -> u64 {
    let mut total = 0_u64;
    for i in 0..iterations {
        name!(value = black_box(i));
        if let Some(proof) = positive::check(&value) {
            let _ = black_box(proof);
            total = total.wrapping_add(black_box(*value.value()));
        }
    }
    black_box(total)
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u64>().expect("integer iteration count"))
        .unwrap_or(5_000_000);
    let mut a = Vec::new();
    let mut b = Vec::new();
    for sample in 0..9 {
        let measure = |f: fn(u64) -> u64, samples: &mut Vec<u128>| {
            let start = Instant::now();
            let result = f(iterations);
            samples.push(start.elapsed().as_nanos());
            let expected = u128::from(iterations) * u128::from(iterations.saturating_sub(1)) / 2;
            assert_eq!(result, expected as u64);
        };
        if sample % 2 == 0 {
            measure(plain, &mut a);
            measure(proved, &mut b);
        } else {
            measure(proved, &mut b);
            measure(plain, &mut a);
        }
    }
    a.sort_unstable();
    b.sort_unstable();
    println!(
        "iterations={iterations} samples=9 plain_median_ns={} proof_median_ns={}",
        a[4], b[4]
    );
}
