pub trait HyperLogLog {
    fn new() -> Self;
    fn add(&mut self, hash: u64);
    fn maybe_contains(&self, hash: u64) -> bool;
    fn cardinality(&self) -> u64;
}

pub trait Merge<Rhs = Self> {
    type Output;

    fn merge(self, rhs: Rhs) -> Self::Output;
}
