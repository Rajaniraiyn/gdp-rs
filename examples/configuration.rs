//! Share a const predicate between static configuration and scoped checks.
use ghostproof::{Named, const_assert, with_names};

struct Config {
    batch: usize,
}

mod policy {
    use super::*;

    pub const fn valid_batch(max: usize, batch: usize) -> bool {
        batch > 0 && batch <= max
    }

    #[ghostproof::proof(subjects(config))]
    pub struct Usable<'config, const MAX: usize> {
        batch: usize,
    }

    pub fn check<'c, const MAX: usize>(config: &Named<'c, Config>) -> Option<Usable<'c, MAX>> {
        let batch = config.value().batch;
        valid_batch(MAX, batch).then(|| Usable::issue(config, batch))
    }

    impl<const MAX: usize> UsableCapability<'_, MAX, Config> {
        pub fn count_batches(&self, data: &[u8]) -> usize {
            data.chunks(self.proof().batch).count()
        }
    }
}

const MAX_BATCH: usize = 128;
const BATCH: usize = 64;
const_assert!(
    policy::valid_batch(MAX_BATCH, BATCH),
    "invalid default batch size"
);

fn main() {
    let count = with_names!(config = Config { batch: BATCH }; {
        policy::check::<MAX_BATCH>(&config).unwrap().bind(config).count_batches(&[0; 130])
    });
    assert_eq!(count, 3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_configuration_uses_the_same_predicate() {
        for batch in [0, 1, 64, 128, 129] {
            with_names!(config = Config { batch }; {
                let checked = policy::check::<MAX_BATCH>(&config);
                assert_eq!(checked.is_some(), policy::valid_batch(MAX_BATCH, batch));
                if let Some(proof) = checked {
                    let capability = proof.bind(config);
                    assert_eq!(capability.config().value().batch, batch);
                    assert_eq!(capability.count_batches(&[]), 0);
                    assert_eq!(capability.count_batches(&[0; 130]), 130_usize.div_ceil(batch));
                }
            });
        }
    }

    #[test]
    fn zero_maximum_rejects_all_batch_sizes() {
        for batch in [0, 1, usize::MAX] {
            with_names!(config = Config { batch }; {
                assert!(policy::check::<0>(&config).is_none());
            });
        }
    }
}
