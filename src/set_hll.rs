use crate::{HyperLogLog, Merge};
use std::collections::BTreeSet;

pub struct SetHyperLogLog {
    pub(crate) set: BTreeSet<u64>,
}

impl HyperLogLog for SetHyperLogLog {
    fn new() -> Self {
        SetHyperLogLog {
            set: BTreeSet::new(),
        }
    }

    fn add(&mut self, hash: u64) {
        self.set.insert(hash);
    }

    fn maybe_contains(&self, hash: u64) -> bool {
        self.set.contains(&hash)
    }

    fn cardinality(&self) -> u64 {
        self.set.len() as u64
    }
}

impl Merge<SetHyperLogLog> for SetHyperLogLog {
    type Output = SetHyperLogLog;

    fn merge(mut self, mut rhs: SetHyperLogLog) -> Self::Output {
        if self.set.len() >= rhs.set.len() {
            self.set.append(&mut rhs.set);
            self
        } else {
            rhs.set.append(&mut self.set);
            rhs
        }
    }
}
