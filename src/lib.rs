mod constants;
mod dense_hll;
mod hyperloglog;
mod set_hll;
mod sparse_hll;

pub use dense_hll::DenseHyperLogLog;
pub use hyperloglog::{HyperLogLog, Merge};
pub use set_hll::SetHyperLogLog;
pub use sparse_hll::SparseHyperLogLog;

pub(crate) fn linear_counting(total_buckets: f64, zero_buckets: f64) -> u64 {
    (total_buckets * (total_buckets / zero_buckets).ln()).round() as u64
}
