/// Summary of one target of a benchmark.
#[derive(Debug, PartialEq)]
pub(crate) struct TargetSummary {
    /// Median nanoseconds per iteration.
    pub(crate) time: f64,
    /// Quartiles of the per-sample ratios to the baseline, `None` for the
    /// baseline itself.
    pub(crate) ratio: Option<Quartiles>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Quartiles {
    pub(crate) low: f64,
    pub(crate) median: f64,
    pub(crate) high: f64,
}

/// Summarizes the non-empty `samples`, each holding the nanoseconds per
/// iteration of every target measured in turn, the first target being the
/// baseline.
///
/// Ratios within a sample cancel out drift (CPU frequency, caches,
/// background load) affecting all of its targets.
pub(crate) fn summarize(samples: &[Vec<f64>]) -> Vec<TargetSummary> {
    (0..samples[0].len())
        .map(|target| TargetSummary {
            time: percentile(&sorted(samples.iter().map(|s| s[target]).collect()), 50),
            ratio: (target > 0).then(|| {
                let ratios = sorted(samples.iter().map(|s| s[target] / s[0]).collect());

                Quartiles {
                    low: percentile(&ratios, 25),
                    median: percentile(&ratios, 50),
                    high: percentile(&ratios, 75),
                }
            }),
        })
        .collect()
}

fn sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(f64::total_cmp);
    values
}

/// Returns the nearest-rank `p`th percentile of the non-empty `sorted`
/// values.
fn percentile(sorted: &[f64], p: usize) -> f64 {
    sorted[(sorted.len() - 1) * p / 100]
}

#[cfg(test)]
mod tests {
    use super::*;

    mod summarize {
        use super::*;

        #[test]
        fn summarize_should_return_medians_and_ratio_quartiles_to_baseline() {
            let samples = [
                vec![100.0, 150.0, 50.0],
                vec![200.0, 220.0, 100.0],
                vec![100.0, 100.0, 70.0],
                vec![100.0, 130.0, 40.0],
                vec![400.0, 480.0, 280.0],
            ];

            assert_eq!(
                summarize(&samples),
                vec![
                    TargetSummary {
                        time: 100.0,
                        ratio: None,
                    },
                    TargetSummary {
                        time: 150.0,
                        ratio: Some(Quartiles {
                            low: 1.1,
                            median: 1.2,
                            high: 1.3,
                        }),
                    },
                    TargetSummary {
                        time: 70.0,
                        ratio: Some(Quartiles {
                            low: 0.5,
                            median: 0.5,
                            high: 0.7,
                        }),
                    },
                ]
            );
        }
    }

    mod percentile {
        use super::*;

        #[test]
        fn percentile_should_return_nearest_rank_value() {
            let sorted = [1.0, 2.0, 3.0, 4.0];

            for (p, expected) in [(0, 1.0), (25, 1.0), (50, 2.0), (75, 3.0), (100, 4.0)] {
                assert_eq!(percentile(&sorted, p), expected, "p: {p}");
            }
        }
    }
}
