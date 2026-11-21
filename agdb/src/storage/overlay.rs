use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Bound::Excluded;
use std::ops::Bound::Unbounded;

/// Memory of a range beyond its bytes: its share of half-full map nodes
/// and the smallest allocation, measured with glibc.
pub(crate) const RUN_OVERHEAD: u64 = 96;

/// Bytes written over a file held as disjoint ranges by their start. Ranges
/// grow only at their end so that no write copies the bytes held before it.
#[derive(Debug, Default)]
pub(crate) struct Overlay {
    ranges: BTreeMap<u64, Vec<u8>>,
    size: u64,
}

impl Overlay {
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.size = 0;
    }

    pub(crate) fn get(&self, pos: u64, end: u64) -> Option<&[u8]> {
        let (start, bytes) = self.ranges.range(..=pos).next_back()?;
        let to = end - start;
        (to <= bytes.len() as u64).then(|| &bytes[(pos - start) as usize..to as usize])
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Touching ranges are joined so that each is written at once.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (u64, Cow<'_, [u8]>)> + '_ {
        let mut ranges = self.ranges.iter().peekable();

        std::iter::from_fn(move || {
            let (start, bytes) = ranges.next()?;
            let mut end = start + bytes.len() as u64;
            let mut joined = Cow::Borrowed(bytes.as_slice());

            while let Some((_, next)) = ranges.next_if(|(next, _)| **next == end) {
                joined.to_mut().extend_from_slice(next);
                end += next.len() as u64;
            }

            Some((*start, joined))
        })
    }

    pub(crate) fn patch(&self, pos: u64, buffer: &mut [u8]) {
        let end = pos + buffer.len() as u64;
        let first = self
            .ranges
            .range(..=pos)
            .next_back()
            .map_or(pos, |(start, _)| *start);

        for (start, bytes) in self.ranges.range(first..end) {
            let from = pos.max(*start);
            let to = end.min(start + bytes.len() as u64);

            if from < to {
                buffer[(from - pos) as usize..(to - pos) as usize]
                    .copy_from_slice(&bytes[(from - start) as usize..(to - start) as usize]);
            }
        }
    }

    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn truncate(&mut self, len: u64) {
        for bytes in self.ranges.split_off(&len).values() {
            self.size -= bytes.len() as u64 + RUN_OVERHEAD;
        }

        if let Some((start, bytes)) = self.ranges.iter_mut().next_back() {
            let kept = (len - start).min(bytes.len() as u64);
            self.size -= bytes.len() as u64 - kept;
            bytes.truncate(kept as usize);
        }
    }

    pub(crate) fn write(&mut self, mut pos: u64, mut bytes: &[u8]) {
        let end = pos + bytes.len() as u64;

        while !bytes.is_empty() {
            let next = self
                .ranges
                .range((Excluded(pos), Unbounded))
                .next()
                .map_or(end, |(start, _)| end.min(*start));
            let (part, rest) = bytes.split_at((next - pos) as usize);

            match self.ranges.range_mut(..=pos).next_back() {
                Some((start, range)) if pos <= start + range.len() as u64 => {
                    let from = (pos - start) as usize;
                    let overlap = (range.len() - from).min(part.len());
                    range[from..from + overlap].copy_from_slice(&part[..overlap]);
                    range.extend_from_slice(&part[overlap..]);
                    self.size += (part.len() - overlap) as u64;
                }
                _ => {
                    self.ranges.insert(pos, part.to_vec());
                    self.size += part.len() as u64 + RUN_OVERHEAD;
                }
            }

            pos = next;
            bytes = rest;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(writes: &[(u64, &[u8])]) -> Overlay {
        let mut overlay = Overlay::default();

        for (pos, bytes) in writes {
            overlay.write(*pos, bytes);
        }

        overlay
    }

    fn all(overlay: &Overlay) -> Vec<(u64, Vec<u8>)> {
        overlay
            .ranges
            .iter()
            .map(|(start, bytes)| (*start, bytes.clone()))
            .collect()
    }

    fn joined(overlay: &Overlay) -> Vec<(u64, Vec<u8>)> {
        overlay
            .iter()
            .map(|(start, bytes)| (start, bytes.to_vec()))
            .collect()
    }

    fn patched(overlay: &Overlay, pos: u64, len: usize) -> Vec<u8> {
        let mut buffer = vec![0; len];
        overlay.patch(pos, &mut buffer);
        buffer
    }

    mod overlay {
        use super::*;

        mod clear {
            use super::*;

            #[test]
            fn clear_should_remove_all_bytes() {
                let mut overlay = overlay(&[(1, &[1]), (4, &[4, 5])]);

                overlay.clear();

                assert!(overlay.is_empty());
                assert_eq!(overlay.size(), 0);
            }
        }

        mod get {
            use super::*;

            #[test]
            fn get_should_return_bytes_within_one_range() {
                let overlay = overlay(&[(2, &[1, 2, 3, 4]), (10, &[5])]);

                assert_eq!(overlay.get(2, 6), Some(&[1, 2, 3, 4][..]));
                assert_eq!(overlay.get(3, 5), Some(&[2, 3][..]));
                assert_eq!(overlay.get(10, 11), Some(&[5][..]));
            }

            #[test]
            fn get_should_not_return_bytes_partially_outside_range() {
                let overlay = overlay(&[(2, &[1, 2, 3, 4]), (10, &[5])]);

                assert_eq!(overlay.get(1, 3), None);
                assert_eq!(overlay.get(5, 7), None);
                assert_eq!(overlay.get(5, 11), None);
                assert_eq!(overlay.get(6, 7), None);
            }

            #[test]
            fn get_empty_should_return_nothing() {
                assert_eq!(Overlay::default().get(0, 1), None);
            }
        }

        mod is_empty {
            use super::*;

            #[test]
            fn is_empty_should_tell_whether_there_are_any_bytes() {
                assert!(Overlay::default().is_empty());
                assert!(!overlay(&[(1, &[1])]).is_empty());
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_return_ranges_in_order() {
                let overlay = overlay(&[(7, &[7]), (1, &[1]), (4, &[4])]);

                assert_eq!(joined(&overlay), [(1, vec![1]), (4, vec![4]), (7, vec![7])]);
            }

            #[test]
            fn iter_should_join_touching_ranges() {
                let overlay = overlay(&[(6, &[3]), (4, &[2, 2]), (1, &[1]), (8, &[4])]);

                assert_eq!(
                    joined(&overlay),
                    [(1, vec![1]), (4, vec![2, 2, 3]), (8, vec![4])]
                );
            }
        }

        mod patch {
            use super::*;

            #[test]
            fn patch_should_copy_bytes_of_overlapping_ranges() {
                let overlay = overlay(&[(0, &[1, 1]), (3, &[2]), (5, &[3, 3, 3]), (10, &[4])]);

                assert_eq!(patched(&overlay, 1, 6), [1, 0, 2, 0, 3, 3]);
            }

            #[test]
            fn patch_without_overlapping_ranges_should_keep_buffer() {
                let overlay = overlay(&[(0, &[1]), (5, &[2])]);

                assert_eq!(patched(&overlay, 1, 4), [0, 0, 0, 0]);
            }
        }

        mod size {
            use super::*;

            #[test]
            fn size_should_count_bytes_held_once_and_overhead_of_ranges() {
                let overlay = overlay(&[(0, &[1, 1]), (1, &[2, 2]), (5, &[3])]);

                assert_eq!(overlay.size(), 4 + 2 * RUN_OVERHEAD);
            }
        }

        mod truncate {
            use super::*;

            #[test]
            fn truncate_should_cut_off_bytes_from_len() {
                let mut overlay = overlay(&[(0, &[1, 1]), (4, &[2, 2, 2]), (8, &[3])]);

                overlay.truncate(5);

                assert_eq!(all(&overlay), [(0, vec![1, 1]), (4, vec![2])]);
                assert_eq!(overlay.size(), 3 + 2 * RUN_OVERHEAD);
            }

            #[test]
            fn truncate_past_bytes_should_keep_them() {
                let mut overlay = overlay(&[(0, &[1, 1])]);

                overlay.truncate(5);

                assert_eq!(all(&overlay), [(0, vec![1, 1])]);
                assert_eq!(overlay.size(), 2 + RUN_OVERHEAD);
            }

            #[test]
            fn truncate_at_range_start_should_remove_it() {
                let mut overlay = overlay(&[(0, &[1]), (4, &[2])]);

                overlay.truncate(4);

                assert_eq!(all(&overlay), [(0, vec![1])]);
                assert_eq!(overlay.size(), 1 + RUN_OVERHEAD);
            }
        }

        mod write {
            use super::*;

            #[test]
            fn write_should_keep_disjoint_ranges_ordered() {
                let overlay = overlay(&[(10, &[2]), (2, &[1])]);

                assert_eq!(all(&overlay), [(2, vec![1]), (10, vec![2])]);
            }

            #[test]
            fn write_touching_range_end_should_extend_it() {
                let overlay = overlay(&[(2, &[1, 1]), (4, &[2, 2]), (5, &[3, 3, 3])]);

                assert_eq!(all(&overlay), [(2, vec![1, 1, 2, 3, 3, 3])]);
                assert_eq!(overlay.size(), 6 + RUN_OVERHEAD);
            }

            #[test]
            fn write_touching_range_start_should_keep_it() {
                // WHY: prepending would copy the range on every write before it
                let overlay = overlay(&[(4, &[2, 2]), (2, &[1, 1]), (1, &[3, 3])]);

                assert_eq!(
                    all(&overlay),
                    [(1, vec![3]), (2, vec![3, 1]), (4, vec![2, 2])]
                );
                assert_eq!(overlay.size(), 5 + 3 * RUN_OVERHEAD);
            }

            #[test]
            fn write_should_overwrite_ranges_it_spans_and_fill_gaps_between() {
                let overlay = overlay(&[
                    (2, &[1, 1]),
                    (6, &[2, 2]),
                    (10, &[3, 3]),
                    (20, &[4]),
                    (3, &[5; 8]),
                ]);

                assert_eq!(
                    all(&overlay),
                    [
                        (2, vec![1, 5, 5, 5]),
                        (6, vec![5, 5, 5, 5]),
                        (10, vec![5, 3]),
                        (20, vec![4])
                    ]
                );
                assert_eq!(overlay.size(), 11 + 4 * RUN_OVERHEAD);
            }

            #[test]
            fn write_within_range_should_overwrite_its_bytes() {
                let overlay = overlay(&[(2, &[1, 1, 1, 1]), (3, &[2])]);

                assert_eq!(all(&overlay), [(2, vec![1, 2, 1, 1])]);
                assert_eq!(overlay.size(), 4 + RUN_OVERHEAD);
            }
        }
    }
}
