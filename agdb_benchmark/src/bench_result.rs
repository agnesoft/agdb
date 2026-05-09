use crate::bench_error::BenchError;

pub(crate) type BenchResult<T> = Result<T, BenchError>;
