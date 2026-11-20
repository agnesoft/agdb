use std::collections::BTreeMap;

/// Disjoint byte ranges `[start, end)`, overlapping and touching ones
/// merged into one.
#[derive(Debug, Default)]
pub(crate) struct Ranges {
    ranges: BTreeMap<u64, u64>,
}

impl Ranges {
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
    }

    pub(crate) fn contains(&self, start: u64, end: u64) -> bool {
        self.ranges
            .range(..=start)
            .next_back()
            .is_some_and(|(_, range_end)| end <= *range_end)
    }

    pub(crate) fn insert(&mut self, mut start: u64, mut end: u64) {
        if let Some((range_start, range_end)) = self.ranges.range(..=start).next_back()
            && start <= *range_end
        {
            start = *range_start;
            end = end.max(*range_end);
        }

        while let Some((range_start, range_end)) = self.ranges.range(start..=end).next() {
            let range_start = *range_start;
            end = end.max(*range_end);
            self.ranges.remove(&range_start);
        }

        self.ranges.insert(start, end);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        self.ranges.iter().map(|(start, end)| (*start, *end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges(ranges: &[(u64, u64)]) -> Ranges {
        let mut result = Ranges::default();

        for (start, end) in ranges {
            result.insert(*start, *end);
        }

        result
    }

    fn all(ranges: &Ranges) -> Vec<(u64, u64)> {
        ranges.iter().collect()
    }

    mod ranges {
        use super::*;

        mod clear {
            use super::*;

            #[test]
            fn clear_should_remove_all_ranges() {
                let mut ranges = ranges(&[(1, 2), (4, 5)]);

                ranges.clear();

                assert!(ranges.is_empty());
            }
        }

        mod contains {
            use super::*;

            #[test]
            fn contains_should_find_range_within_one_range() {
                let ranges = ranges(&[(2, 6), (10, 12)]);

                assert!(ranges.contains(2, 6));
                assert!(ranges.contains(3, 5));
                assert!(ranges.contains(10, 11));
            }

            #[test]
            fn contains_should_not_find_range_partially_outside() {
                let ranges = ranges(&[(2, 6), (10, 12)]);

                assert!(!ranges.contains(1, 3));
                assert!(!ranges.contains(5, 7));
                assert!(!ranges.contains(5, 11));
                assert!(!ranges.contains(6, 7));
            }

            #[test]
            fn contains_empty_should_find_nothing() {
                assert!(!Ranges::default().contains(0, 1));
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_keep_disjoint_ranges_ordered() {
                let ranges = ranges(&[(10, 12), (2, 4)]);

                assert_eq!(all(&ranges), [(2, 4), (10, 12)]);
            }

            #[test]
            fn insert_should_merge_overlapping_and_touching_ranges() {
                let ranges = ranges(&[(2, 4), (4, 6), (5, 8), (1, 3)]);

                assert_eq!(all(&ranges), [(1, 8)]);
            }

            #[test]
            fn insert_should_merge_all_ranges_it_spans() {
                let ranges = ranges(&[(2, 4), (6, 8), (10, 12), (20, 22), (3, 11)]);

                assert_eq!(all(&ranges), [(2, 12), (20, 22)]);
            }

            #[test]
            fn insert_within_range_should_keep_it() {
                let ranges = ranges(&[(2, 10), (3, 5)]);

                assert_eq!(all(&ranges), [(2, 10)]);
            }
        }

        mod is_empty {
            use super::*;

            #[test]
            fn is_empty_should_tell_whether_there_is_any_range() {
                assert!(Ranges::default().is_empty());
                assert!(!ranges(&[(1, 2)]).is_empty());
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_return_ranges_in_order() {
                let ranges = ranges(&[(7, 9), (1, 2), (4, 5)]);

                assert_eq!(all(&ranges), [(1, 2), (4, 5), (7, 9)]);
            }
        }
    }
}
