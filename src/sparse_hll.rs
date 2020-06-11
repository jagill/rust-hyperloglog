use crate::constants::SPARSE_PREFIX_LENGTH;
use crate::{HyperLogLog, Merge, SetHyperLogLog};
use std::collections::BTreeMap;

pub struct SparseHyperLogLog {
    pub(crate) buckets: BTreeMap<u32, u8>,
}

impl HyperLogLog for SparseHyperLogLog {
    fn new() -> Self {
        SparseHyperLogLog {
            buckets: BTreeMap::new(),
        }
    }

    fn add(&mut self, hash: u64) {
        self.insert(self.find_bucket(hash), self.find_value(hash));
    }

    fn maybe_contains(&self, hash: u64) -> bool {
        self.buckets.contains_key(&self.find_bucket(hash))
    }

    fn cardinality(&self) -> u64 {
        let total_buckets = (1u64 << SPARSE_PREFIX_LENGTH) as f64;
        let zero_buckets = total_buckets - (self.buckets.len() as f64);
        (total_buckets * (total_buckets / zero_buckets).ln()).round() as u64
    }
}

impl SparseHyperLogLog {
    fn find_bucket(&self, hash: u64) -> u32 {
        (hash >> (64 - SPARSE_PREFIX_LENGTH)) as u32
    }

    fn find_value(&self, hash: u64) -> u8 {
        (64 - SPARSE_PREFIX_LENGTH).min(hash.trailing_zeros()) as u8
    }

    pub(crate) fn insert(&mut self, bucket: u32, value: u8) {
        self.buckets
            .entry(bucket)
            .and_modify(|e| *e = value.max(*e))
            .or_insert(value);
    }

    pub(crate) fn get_reduced_bucket(&self, bucket: u32, prefix_length: u32) -> u32 {
        let excess_prefix_length = SPARSE_PREFIX_LENGTH - prefix_length;
        bucket >> excess_prefix_length
    }

    pub(crate) fn get_total_zeros(&self, bucket: u32, prefix_length: u32) -> u8 {
        let excess_prefix_length = SPARSE_PREFIX_LENGTH - prefix_length;
        let prefix_stop_bit = 1 << (excess_prefix_length + 1);
        let sparse_value_bits = (64 - SPARSE_PREFIX_LENGTH) as u8;
        let mut total_zeros = self.buckets[&bucket];
        if total_zeros == sparse_value_bits {
            let zeros_in_prefix = (prefix_stop_bit & bucket).trailing_zeros() as u8;
            total_zeros += zeros_in_prefix;
        }
        total_zeros
    }
}

impl From<SetHyperLogLog> for SparseHyperLogLog {
    fn from(set_hll: SetHyperLogLog) -> Self {
        let mut sparse_hll = SparseHyperLogLog::new();
        for hash in set_hll.set.into_iter() {
            sparse_hll.add(hash)
        }
        sparse_hll
    }
}

impl Merge<SparseHyperLogLog> for SparseHyperLogLog {
    type Output = SparseHyperLogLog;

    fn merge(mut self, mut rhs: SparseHyperLogLog) -> Self::Output {
        if self.buckets.len() >= rhs.buckets.len() {
            for (b, v) in rhs.buckets.into_iter() {
                self.insert(b, v);
            }
            self
        } else {
            for (b, v) in self.buckets.into_iter() {
                rhs.insert(b, v);
            }
            rhs
        }
    }
}

impl Merge<SetHyperLogLog> for SparseHyperLogLog {
    type Output = SparseHyperLogLog;

    fn merge(mut self, rhs: SetHyperLogLog) -> Self::Output {
        for x in rhs.set.into_iter() {
            self.add(x);
        }
        self
    }
}

impl Merge<SparseHyperLogLog> for SetHyperLogLog {
    type Output = SparseHyperLogLog;

    fn merge(self, rhs: SparseHyperLogLog) -> Self::Output {
        rhs.merge(self)
    }
}
