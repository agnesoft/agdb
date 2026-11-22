use crate::DbError;
use crate::storage::alloc::Allocator;
use crate::storage::pages::ViewMut;

/// 32 segments of `2^k` pages cover any file (`2^32` pages).
const SEGMENTS: usize = 32;
/// The bytes of an encoded [`SegmentedArray`].
pub(crate) const ARRAY_SIZE: usize = 4 * SEGMENTS;

/// Records of `1 << record_shift` bytes in segments that never move, the
/// `k`-th of `2^k` pages, so that growing never copies and a record never
/// crosses a page. The header holds the first page of each segment.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SegmentedArray {
    page_shift: u32,
    record_shift: u32,
    segments: [u32; SEGMENTS],
}

impl SegmentedArray {
    pub(crate) fn new(page_shift: u32, record_shift: u32) -> Self {
        Self {
            page_shift,
            record_shift,
            segments: [0; SEGMENTS],
        }
    }

    /// Reads the segments of an array of the same shifts.
    pub(crate) fn decode(&mut self, bytes: &[u8]) {
        for (segment, word) in self.segments.iter_mut().zip(bytes.as_chunks::<4>().0) {
            *segment = u32::from_le_bytes(*word);
        }
    }

    pub(crate) fn encode(&self) -> [u8; ARRAY_SIZE] {
        let mut bytes = [0_u8; ARRAY_SIZE];

        for (chunk, segment) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(self.segments) {
            *chunk = segment.to_le_bytes();
        }

        bytes
    }

    pub(crate) fn record_size(&self) -> u64 {
        1 << self.record_shift
    }

    pub(crate) fn capacity(&self) -> u64 {
        ((1 << self.len()) - 1) << self.base_shift()
    }

    /// The position of the record `slot`, which must be below the
    /// capacity.
    pub(crate) fn pos(&self, slot: u64) -> u64 {
        let index = slot + (1 << self.base_shift());
        let top = index.ilog2();
        let page = u64::from(self.segments[(top - self.base_shift()) as usize]);
        (page << self.page_shift) + ((index - (1 << top)) << self.record_shift)
    }

    /// Appends the next segment, doubling the capacity plus a page of
    /// records.
    pub(crate) fn grow(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
    ) -> Result<(), DbError> {
        let segment = self.len();
        let pos = alloc.alloc(view, 1 << (self.page_shift + segment as u32))?;
        self.segments[segment] = (pos >> self.page_shift) as u32;
        Ok(())
    }

    /// The segments in use, which hold a page past the root.
    fn len(&self) -> usize {
        self.segments
            .iter()
            .position(|page| *page == 0)
            .unwrap_or(SEGMENTS)
    }

    /// The records of a page.
    fn base_shift(&self) -> u32 {
        self.page_shift - self.record_shift
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::StorageSlice;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::MAX_PAGE_SHIFT;
    use crate::storage::pages::View;
    use std::collections::BTreeMap;

    /// The smallest page, so that the length of the file is a number of
    /// chunks.
    const CHUNK_SHIFT: u32 = DEFAULT_PAGE_SHIFT;
    const CHUNK: u64 = 1 << CHUNK_SHIFT;

    /// A view of zero bytes that keeps only the chunks written to it, to
    /// run files of terabytes.
    #[derive(Default)]
    pub(crate) struct Sparse {
        len: u64,
        chunks: BTreeMap<u64, Vec<u8>>,
    }

    impl View for Sparse {
        fn len(&self) -> u64 {
            self.len
        }

        fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
            assert!(pos + len <= self.len, "read {pos} + {len}");
            let mut bytes = vec![0; len as usize];
            let mut at = pos;

            while at < pos + len {
                let offset = (at % CHUNK) as usize;
                let size = (CHUNK - at % CHUNK).min(pos + len - at) as usize;

                if let Some(chunk) = self.chunks.get(&(at >> CHUNK_SHIFT)) {
                    bytes[(at - pos) as usize..][..size]
                        .copy_from_slice(&chunk[offset..offset + size]);
                }

                at += size as u64;
            }

            Ok(StorageSlice::Owned(bytes))
        }
    }

    impl ViewMut for Sparse {
        fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
            assert!(pos + bytes.len() as u64 <= self.len, "write {pos}");
            let mut at = pos;
            let mut rest = bytes;

            while !rest.is_empty() {
                let offset = (at % CHUNK) as usize;
                let size = (CHUNK as usize - offset).min(rest.len());
                let chunk = self
                    .chunks
                    .entry(at >> CHUNK_SHIFT)
                    .or_insert_with(|| vec![0; CHUNK as usize]);
                chunk[offset..offset + size].copy_from_slice(&rest[..size]);
                at += size as u64;
                rest = &rest[size..];
            }

            Ok(())
        }

        fn set_len(&mut self, len: u64) {
            self.chunks.retain(|chunk, _| *chunk << CHUNK_SHIFT < len);
            self.len = len;
        }
    }

    /// The sparse view of a file holding only the root.
    pub(crate) fn sparse(alloc: &Allocator) -> Sparse {
        Sparse {
            len: alloc.len(),
            chunks: BTreeMap::new(),
        }
    }

    /// The (pos, size) of each segment.
    pub(crate) fn allocations(array: &SegmentedArray) -> Vec<(u64, u64)> {
        (0..array.len())
            .map(|segment| {
                let page_size = 1 << array.page_shift;
                (
                    u64::from(array.segments[segment]) * page_size,
                    page_size << segment,
                )
            })
            .collect()
    }

    mod segmented_array {
        use super::*;

        #[test]
        fn segmented_array_should_keep_records_in_place_as_it_grows() {
            with_view(|view, alloc| {
                let mut array = SegmentedArray::new(DEFAULT_PAGE_SHIFT, 6);
                let mut written = 0;
                let mut live = Vec::new();

                for segments in 1..=6 {
                    array.grow(view, alloc).unwrap();
                    live.push((alloc.alloc(view, 100).unwrap(), 100));
                    assert_eq!(array.capacity(), ((1 << segments) - 1) * 16);

                    for slot in written..array.capacity() {
                        view.write(array.pos(slot), &slot.to_le_bytes()).unwrap();
                    }

                    written = array.capacity();
                }

                for slot in 0..written {
                    let bytes = view.read(array.pos(slot), 8).unwrap();
                    assert_eq!(u64::from_le_bytes(bytes[..].try_into().unwrap()), slot);
                }

                live.extend(allocations(&array));
                verify(alloc, view, &live);
            });
        }

        mod decode {
            use super::*;

            #[test]
            fn decode_should_read_encoded_array() {
                let mut array = SegmentedArray::new(12, 5);
                array.segments[..3].copy_from_slice(&[7, 9, 3]);
                let mut decoded = SegmentedArray::new(12, 5);
                decoded.decode(&array.encode());
                assert_eq!(decoded, array);
                assert_eq!((decoded.capacity(), decoded.record_size()), (7 * 128, 32));
            }
        }

        mod pos {
            use super::*;

            #[test]
            fn pos_should_map_slot_to_its_segment_and_offset() {
                let mut array = SegmentedArray::new(DEFAULT_PAGE_SHIFT, 5);
                array.segments[..4].copy_from_slice(&[10, 20, 40, 80]);
                assert_eq!(array.pos(0), 10 * 1024);
                assert_eq!(array.pos(31), 10 * 1024 + 31 * 32);
                assert_eq!(array.pos(32), 20 * 1024);
                assert_eq!(array.pos(95), 21 * 1024 + 31 * 32);
                assert_eq!(array.pos(96), 40 * 1024);
                assert_eq!(array.pos(223), 43 * 1024 + 31 * 32);
                assert_eq!(array.pos(224), 80 * 1024);
                assert_eq!(array.pos(479), 87 * 1024 + 31 * 32);
            }
        }

        mod grow {
            use super::*;

            #[test]
            fn grow_failure_should_fail() {
                assert_failures(
                    |view, alloc| {
                        let pos = alloc.alloc(view, 3 * 1024).unwrap();
                        alloc.alloc(view, 1024).unwrap();
                        alloc.free(view, pos, 3 * 1024).unwrap();
                        SegmentedArray::new(DEFAULT_PAGE_SHIFT, 5)
                    },
                    |view, alloc, mut array| {
                        array.grow(view, alloc)?;
                        array.grow(view, alloc)?;
                        array.grow(view, alloc)
                    },
                );
            }

            #[test]
            fn grow_past_storage_should_fail() {
                for shift in [DEFAULT_PAGE_SHIFT, MAX_PAGE_SHIFT] {
                    let mut alloc = Allocator::new(shift);
                    let mut view = sparse(&alloc);
                    let mut array = SegmentedArray::new(shift, shift);

                    let mut result = Ok(());

                    while result.is_ok() {
                        result = array.grow(&mut view, &mut alloc);
                    }

                    let error = result.unwrap_err();
                    assert!(error.description.starts_with("Storage full"), "{error}");
                    assert_eq!(array.len(), 31);
                    assert_eq!(array.capacity(), (1 << 31) - 1);
                    let last = array.pos(array.capacity() - 1);
                    assert_eq!(last + (1 << shift), view.len());
                    assert_eq!(view.read(last, 2).unwrap()[..], [0, 0]);
                    view.write(last, &[1]).unwrap();
                    assert_eq!(view.read(last, 2).unwrap()[..], [1, 0]);
                }
            }
        }
    }
}
