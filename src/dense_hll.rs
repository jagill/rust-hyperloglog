use crate::constants::DEFAULT_PREFIX_LENGTH;
use crate::{HyperLogLog, Merge, SetHyperLogLog, SparseHyperLogLog};

pub struct DenseHyperLogLog {
    prefix_length: u32,
    buckets: Vec<u8>,
}

impl HyperLogLog for DenseHyperLogLog {
    fn new() -> Self {
        let prefix_length = DEFAULT_PREFIX_LENGTH;
        assert!(prefix_length <= 32);
        let bucket_count = 1 << prefix_length as usize;
        DenseHyperLogLog {
            prefix_length,
            buckets: Vec::with_capacity(bucket_count),
        }
    }

    fn add(&mut self, hash: u64) {
        self.insert(self.find_bucket(hash), self.find_value(hash));
    }

    fn maybe_contains(&self, hash: u64) -> bool {
        self.buckets[self.find_bucket(hash) as usize] >= self.find_value(hash)
    }

    fn cardinality(&self) -> u64 {
        // TODO: STUB
        self.num_buckets() as u64
    }
}

impl DenseHyperLogLog {
    fn num_buckets(&self) -> usize {
        self.buckets.len() as usize
    }

    fn find_bucket(&self, hash: u64) -> u32 {
        (hash >> (64 - self.prefix_length)) as u32
    }

    fn find_value(&self, hash: u64) -> u8 {
        let num_zeros = hash.trailing_zeros().min(64 - self.prefix_length);
        (num_zeros + 1) as u8
    }

    fn get_value(&self, bucket: u32) -> u8 {
        self.buckets[bucket as usize]
    }

    pub(crate) fn insert(&mut self, bucket: u32, value: u8) {
        let index = bucket as usize;
        self.buckets[index] = value.max(self.buckets[index]);
    }
}

impl From<SetHyperLogLog> for DenseHyperLogLog {
    fn from(set_hll: SetHyperLogLog) -> Self {
        let mut dense_hll = DenseHyperLogLog::new();
        for hash in set_hll.set.into_iter() {
            dense_hll.add(hash)
        }
        dense_hll
    }
}

impl From<SparseHyperLogLog> for DenseHyperLogLog {
    fn from(sparse_hll: SparseHyperLogLog) -> Self {
        let mut dense_hll = DenseHyperLogLog::new();

        for full_bucket in sparse_hll.buckets.keys().cloned() {
            let bucket = sparse_hll.get_reduced_bucket(full_bucket, dense_hll.prefix_length);
            let zeros = sparse_hll.get_total_zeros(full_bucket, dense_hll.prefix_length);
            // Store the number of zeroes + 1, to record an entry with a trailing 1
            dense_hll.insert(bucket, zeros + 1);
        }
        dense_hll
    }
}

impl Merge<DenseHyperLogLog> for DenseHyperLogLog {
    type Output = DenseHyperLogLog;

    fn merge(mut self, rhs: DenseHyperLogLog) -> Self::Output {
        assert!(self.buckets.len() == rhs.buckets.len());
        for (b, v) in rhs.buckets.into_iter().enumerate() {
            self.buckets[b] = self.buckets[b].max(v);
        }
        self
    }
}

impl Merge<SparseHyperLogLog> for DenseHyperLogLog {
    type Output = DenseHyperLogLog;

    fn merge(self, rhs: SparseHyperLogLog) -> Self::Output {
        self.merge(DenseHyperLogLog::from(rhs))
    }
}

impl Merge<DenseHyperLogLog> for SparseHyperLogLog {
    type Output = DenseHyperLogLog;

    fn merge(self, rhs: DenseHyperLogLog) -> Self::Output {
        rhs.merge(self)
    }
}

impl Merge<SetHyperLogLog> for DenseHyperLogLog {
    type Output = DenseHyperLogLog;

    fn merge(mut self, rhs: SetHyperLogLog) -> Self::Output {
        for hash in rhs.set {
            self.add(hash)
        }
        self
    }
}

impl Merge<DenseHyperLogLog> for SetHyperLogLog {
    type Output = DenseHyperLogLog;

    fn merge(self, rhs: DenseHyperLogLog) -> Self::Output {
        rhs.merge(self)
    }
}
