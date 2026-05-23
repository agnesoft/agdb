use std::time::Duration;

#[derive(Clone)]
pub(crate) enum TargetKind {
    Embedded,
    LocalServer,
    RemoteServer,
}

impl TargetKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::LocalServer => "local",
            Self::RemoteServer => "remote",
        }
    }
}

#[derive(Clone)]
pub(crate) struct MemoryStats {
    pub(crate) start: u64,
    pub(crate) peak: u64,
    pub(crate) end: u64,
    pub(crate) end_plus_delay: u64,
}

#[derive(Clone)]
pub(crate) struct TimingStats {
    pub(crate) count: u64,
    pub(crate) min: Duration,
    pub(crate) max: Duration,
    pub(crate) sum_nanos: u128,
}

#[derive(Clone)]
pub(crate) struct WorkloadStats {
    pub(crate) post_writers: TimingStats,
    pub(crate) comment_writers: TimingStats,
    pub(crate) post_readers: TimingStats,
    pub(crate) comment_readers: TimingStats,
}

impl WorkloadStats {
    pub(crate) fn empty() -> Self {
        Self {
            post_writers: TimingStats::empty(),
            comment_writers: TimingStats::empty(),
            post_readers: TimingStats::empty(),
            comment_readers: TimingStats::empty(),
        }
    }
}

impl TimingStats {
    pub(crate) fn empty() -> Self {
        Self {
            count: 0,
            min: Duration::default(),
            max: Duration::default(),
            sum_nanos: 0,
        }
    }

    pub(crate) fn from_times(times: &[Duration]) -> Self {
        if times.is_empty() {
            return Self::empty();
        }

        let mut min = times[0];
        let mut max = times[0];
        let mut sum_nanos: u128 = 0;

        for time in times {
            if *time < min {
                min = *time;
            }
            if *time > max {
                max = *time;
            }
            sum_nanos = sum_nanos.saturating_add(time.as_nanos());
        }

        Self {
            count: times.len() as u64,
            min,
            max,
            sum_nanos,
        }
    }

    pub(crate) fn average(&self) -> Duration {
        if self.count == 0 {
            return Duration::default();
        }

        let avg_nanos = self.sum_nanos / self.count as u128;
        let secs = (avg_nanos / 1_000_000_000u128) as u64;
        let nanos = (avg_nanos % 1_000_000_000u128) as u32;
        Duration::new(secs, nanos)
    }
}

#[derive(Clone)]
pub(crate) struct TargetResult {
    pub(crate) kind: TargetKind,
    pub(crate) address: Option<String>,
    pub(crate) total: Duration,
    pub(crate) workload: WorkloadStats,
    pub(crate) database_before: Option<u64>,
    pub(crate) database_after: Option<u64>,
    pub(crate) memory: Option<MemoryStats>,
    pub(crate) error: Option<String>,
}

impl TargetResult {
    pub(crate) fn ok(
        kind: TargetKind,
        address: Option<String>,
        total: Duration,
        workload: WorkloadStats,
        database_before: Option<u64>,
        database_after: Option<u64>,
        memory: Option<MemoryStats>,
    ) -> Self {
        Self {
            kind,
            address,
            total,
            workload,
            database_before,
            database_after,
            memory,
            error: None,
        }
    }

    pub(crate) fn failed(
        kind: TargetKind,
        address: Option<String>,
        total: Duration,
        error: String,
    ) -> Self {
        Self {
            kind,
            address,
            total,
            workload: WorkloadStats::empty(),
            database_before: None,
            database_after: None,
            memory: None,
            error: Some(error),
        }
    }
}
