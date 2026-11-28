use super::StorageSlice;
use std::sync::Arc;

const PAGE_SHIFT: u32 = 10;
const PAGE: usize = 1 << PAGE_SHIFT;
const SEGMENT_SHIFT: u32 = 20;
const SEGMENT: usize = 1 << SEGMENT_SHIFT;
const SEGMENT_PAGES: usize = SEGMENT / PAGE;
const GROUP_SHIFT: u32 = 5;
const GROUP: usize = 1 << GROUP_SHIFT;
const WORDS: usize = SEGMENT_PAGES / 64;
const FOLD_PAGES: usize = 256;
const MAX_DEPTH: u32 = 8;

type Bits = [u64; WORDS];
type Page = Arc<[u8; PAGE]>;
type Group = Arc<Vec<Arc<Segment>>>;

fn has(bits: &Bits, page: usize) -> bool {
    bits[page >> 6] & (1 << (page & 63)) != 0
}

fn set(bits: &mut Bits, page: usize) {
    bits[page >> 6] |= 1 << (page & 63);
}

fn round_to_page(len: usize) -> usize {
    len.div_ceil(PAGE) * PAGE
}

struct Overrides {
    bits: Bits,
    pages: Vec<(u16, Page)>,
    parent: Option<Arc<Overrides>>,
    depth: u32,
    total: usize,
}

impl Overrides {
    fn new(parent: Option<Arc<Overrides>>) -> Self {
        let (bits, depth, total) = parent
            .as_ref()
            .map_or(([0; WORDS], 0, 0), |p| (p.bits, p.depth, p.total));

        Self {
            bits,
            pages: Vec::new(),
            parent,
            depth: depth + 1,
            total,
        }
    }

    fn get(&self, page: usize) -> &[u8; PAGE] {
        let mut node = self;

        loop {
            if let Some((_, bytes)) = node.pages.iter().find(|(p, _)| *p as usize == page) {
                return bytes;
            }

            node = node
                .parent
                .as_deref()
                .expect("overridden page should be in the chain");
        }
    }

    fn write_into(&self, data: &mut [u8]) {
        let mut seen = [0; WORDS];
        let mut node = Some(self);

        while let Some(n) = node {
            for (page, bytes) in &n.pages {
                if !has(&seen, *page as usize) {
                    set(&mut seen, *page as usize);
                    let start = (*page as usize) << PAGE_SHIFT;
                    data[start..start + PAGE].copy_from_slice(&bytes[..]);
                }
            }

            node = n.parent.as_deref();
        }
    }

    fn merged(&self) -> Self {
        let mut pages = Vec::with_capacity(self.total);
        let mut seen = [0; WORDS];
        let mut node = Some(self);

        while let Some(n) = node {
            for (page, bytes) in &n.pages {
                if !has(&seen, *page as usize) {
                    set(&mut seen, *page as usize);
                    pages.push((*page, bytes.clone()));
                }
            }

            node = n.parent.as_deref();
        }

        Self {
            bits: self.bits,
            total: pages.len(),
            pages,
            parent: None,
            depth: 1,
        }
    }
}

#[derive(Clone)]
struct Segment {
    data: Arc<[u8]>,
    overrides: Option<Arc<Overrides>>,
}

impl Segment {
    fn zeroed(len: usize) -> Self {
        Self {
            data: std::iter::repeat_n(0, len).collect(),
            overrides: None,
        }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn overridden(&self, page: usize) -> Option<&[u8; PAGE]> {
        self.overrides
            .as_deref()
            .filter(|o| has(&o.bits, page))
            .map(|o| o.get(page))
    }

    fn page_bytes(&self, offset: usize, len: usize) -> &[u8] {
        let in_page = offset & (PAGE - 1);

        match self.overridden(offset >> PAGE_SHIFT) {
            Some(bytes) => &bytes[in_page..in_page + len],
            None => &self.data[offset..offset + len],
        }
    }

    fn read(&self, offset: usize, len: usize) -> Option<&[u8]> {
        let Some(overrides) = self.overrides.as_deref() else {
            return Some(&self.data[offset..offset + len]);
        };

        let first = offset >> PAGE_SHIFT;
        let last = (offset + len - 1) >> PAGE_SHIFT;

        if (first..=last).all(|page| !has(&overrides.bits, page)) {
            Some(&self.data[offset..offset + len])
        } else if first == last {
            Some(self.page_bytes(offset, len))
        } else {
            None
        }
    }

    fn write(&mut self, offset: usize, bytes: &[u8]) -> usize {
        let len = bytes.len().min(self.capacity() - offset);
        let bytes = &bytes[..len];

        if self.overrides.is_some() && Arc::get_mut(&mut self.data).is_some() {
            self.fold();
        }

        if self.overrides.is_none()
            && let Some(data) = Arc::get_mut(&mut self.data)
        {
            data[offset..offset + len].copy_from_slice(bytes);
            return len;
        }

        let mut done = 0;

        while done < len {
            let at = offset + done;
            let in_page = at & (PAGE - 1);
            let n = (len - done).min(PAGE - in_page);
            self.override_page(at >> PAGE_SHIFT, in_page, &bytes[done..done + n]);
            done += n;
        }

        let overrides = self.overrides.as_deref().expect("pages were overridden");

        if FOLD_PAGES < overrides.total {
            self.fold();
        } else if MAX_DEPTH < overrides.depth {
            self.overrides = Some(Arc::new(overrides.merged()));
        }

        len
    }

    fn override_page(&mut self, page: usize, in_page: usize, bytes: &[u8]) {
        if !self
            .overrides
            .as_mut()
            .is_some_and(|o| Arc::get_mut(o).is_some())
        {
            let parent = self.overrides.take();
            self.overrides = Some(Arc::new(Overrides::new(parent)));
        }

        let overrides = Arc::get_mut(self.overrides.as_mut().expect("overrides were created"))
            .expect("the newest overrides should not be shared");

        if let Some((_, existing)) = overrides
            .pages
            .iter_mut()
            .find(|(p, _)| *p as usize == page)
        {
            Arc::make_mut(existing)[in_page..in_page + bytes.len()].copy_from_slice(bytes);
            return;
        }

        let mut new_page: Page = Arc::new([0; PAGE]);
        let target = Arc::get_mut(&mut new_page).expect("new page should not be shared");

        match overrides.parent.as_deref().filter(|p| has(&p.bits, page)) {
            Some(parent) => target.copy_from_slice(parent.get(page)),
            None => {
                let start = page << PAGE_SHIFT;
                target.copy_from_slice(&self.data[start..start + PAGE]);
            }
        }

        target[in_page..in_page + bytes.len()].copy_from_slice(bytes);
        overrides.pages.push((page as u16, new_page));

        if !has(&overrides.bits, page) {
            set(&mut overrides.bits, page);
            overrides.total += 1;
        }
    }

    fn fold(&mut self) {
        let Some(overrides) = self.overrides.take() else {
            return;
        };
        overrides.write_into(Arc::make_mut(&mut self.data));
    }

    fn grow(&mut self, capacity: usize) {
        let mut grown: Arc<[u8]> = std::iter::repeat_n(0, capacity).collect();
        let data = Arc::get_mut(&mut grown).expect("new data should be unique");
        data[..self.data.len()].copy_from_slice(&self.data);

        if let Some(overrides) = self.overrides.take() {
            overrides.write_into(data);
        }

        self.data = grown;
    }
}

/// The copy-on-write bytes of the in-memory storages, cheap to clone. A
/// clone is a read-only snapshot of the storage: it does not change with
/// the storage, see [`StorageData::snapshot()`](super::StorageData::snapshot).
#[derive(Clone, Default)]
pub struct Segments {
    groups: Arc<Vec<Group>>,
    len: u64,
}

impl Segments {
    /// The number of bytes.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether there are no bytes.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn read(&self, pos: u64, len: u64) -> StorageSlice<'_> {
        assert!(pos.saturating_add(len) <= self.len, "read out of bounds");

        if len == 0 {
            return StorageSlice::Borrowed(&[]);
        }

        let at = pos as usize;
        let n = len as usize;
        let offset = at & (SEGMENT - 1);

        if offset + n <= SEGMENT
            && let Some(bytes) = self.segment(at >> SEGMENT_SHIFT).read(offset, n)
        {
            return StorageSlice::Borrowed(bytes);
        }

        let mut bytes = Vec::with_capacity(n);
        self.chunks(pos, pos + len)
            .for_each(|(_, chunk)| bytes.extend_from_slice(chunk));
        StorageSlice::Owned(bytes)
    }

    pub(crate) fn chunks(&self, pos: u64, end: u64) -> Chunks<'_> {
        Chunks {
            segments: self,
            pos: pos as usize,
            end: (end.min(self.len)) as usize,
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.len as usize);
        self.chunks(0, self.len)
            .for_each(|(_, chunk)| bytes.extend_from_slice(chunk));
        bytes
    }

    pub(crate) fn holders(&self) -> usize {
        Arc::strong_count(&self.groups)
    }

    pub(crate) fn write(&mut self, pos: u64, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        let end = pos + bytes.len() as u64;

        if self.len < end {
            self.grow(end);
        }

        let mut done = 0;

        while done < bytes.len() {
            let at = pos as usize + done;
            done += self
                .segment_mut(at >> SEGMENT_SHIFT)
                .write(at & (SEGMENT - 1), &bytes[done..]);
        }
    }

    pub(crate) fn resize(&mut self, new_len: u64) {
        if self.len < new_len {
            self.grow(new_len);
        } else if new_len < self.len {
            self.shrink(new_len);
        }
    }

    fn segment_count(&self) -> usize {
        self.groups
            .last()
            .map_or(0, |last| (self.groups.len() - 1) * GROUP + last.len())
    }

    fn segment(&self, index: usize) -> &Segment {
        &self.groups[index >> GROUP_SHIFT][index & (GROUP - 1)]
    }

    fn segment_mut(&mut self, index: usize) -> &mut Segment {
        let groups = Arc::make_mut(&mut self.groups);
        let group = Arc::make_mut(&mut groups[index >> GROUP_SHIFT]);
        Arc::make_mut(&mut group[index & (GROUP - 1)])
    }

    fn capacity(&self) -> usize {
        match self.segment_count() {
            0 => 0,
            count => (count - 1) * SEGMENT + self.segment(count - 1).capacity(),
        }
    }

    fn grow(&mut self, end: u64) {
        // WHY: the bytes past the length are always zero (new ones are, `shrink()` zeroes
        // those it cuts off), so growing needs no fill.
        self.reserve(end as usize);
        self.len = end;
    }

    fn reserve(&mut self, end: usize) {
        let needed = end.div_ceil(SEGMENT);
        let count = self.segment_count();

        if let Some(last) = count.checked_sub(1) {
            let last_end = if count < needed {
                SEGMENT
            } else {
                end - last * SEGMENT
            };
            let capacity = self.segment(last).capacity();

            if capacity < last_end {
                let grown = if last == 0 {
                    round_to_page(last_end.max(capacity * 2)).min(SEGMENT)
                } else {
                    SEGMENT
                };
                self.segment_mut(last).grow(grown);
            }
        }

        for index in count..needed {
            let capacity = if index == 0 {
                round_to_page(end).min(SEGMENT)
            } else {
                SEGMENT
            };
            self.push(Segment::zeroed(capacity));
        }
    }

    fn push(&mut self, segment: Segment) {
        let groups = Arc::make_mut(&mut self.groups);

        match groups.last_mut() {
            Some(group) if group.len() < GROUP => Arc::make_mut(group).push(Arc::new(segment)),
            _ => groups.push(Arc::new(vec![Arc::new(segment)])),
        }
    }

    fn shrink(&mut self, new_len: u64) {
        let needed = (new_len as usize).div_ceil(SEGMENT);
        let old_len = self.len;
        self.len = new_len;

        if needed != self.segment_count() {
            let groups = Arc::make_mut(&mut self.groups);
            groups.truncate(needed.div_ceil(GROUP));

            if let Some(last) = groups.last_mut() {
                Arc::make_mut(last).truncate(needed - (needed - 1) / GROUP * GROUP);
            }
        }

        let end = old_len.min(self.capacity() as u64);
        let mut pos = new_len;

        while pos < end {
            let at = pos as usize;
            let n = (end - pos).min(PAGE as u64) as usize;
            pos += self
                .segment_mut(at >> SEGMENT_SHIFT)
                .write(at & (SEGMENT - 1), &[0; PAGE][..n]) as u64;
        }
    }
}

impl From<&[u8]> for Segments {
    fn from(bytes: &[u8]) -> Self {
        let loaded = Self::load(bytes.len() as u64, |buffer, pos| {
            buffer.copy_from_slice(&bytes[pos as usize..pos as usize + buffer.len()]);
            Ok::<(), std::convert::Infallible>(())
        });

        match loaded {
            Ok(segments) => segments,
        }
    }
}

impl Segments {
    pub(crate) fn load<E>(
        len: u64,
        mut read: impl FnMut(&mut [u8], u64) -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut segments = Self::default();
        let mut pos = 0;

        while pos < len {
            let chunk = (len - pos).min(SEGMENT as u64) as usize;
            let mut data: Arc<[u8]> = std::iter::repeat_n(0, round_to_page(chunk)).collect();
            let bytes = Arc::get_mut(&mut data).expect("new data should be unique");
            read(&mut bytes[..chunk], pos)?;
            segments.push(Segment {
                data,
                overrides: None,
            });
            pos += chunk as u64;
        }

        segments.len = len;
        Ok(segments)
    }
}

pub(crate) struct Chunks<'a> {
    segments: &'a Segments,
    pos: usize,
    end: usize,
}

impl<'a> Iterator for Chunks<'a> {
    type Item = (u64, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.end <= self.pos {
            return None;
        }

        let pos = self.pos;
        let segment = self.segments.segment(pos >> SEGMENT_SHIFT);
        let offset = pos & (SEGMENT - 1);
        let to_segment_end = (self.end - pos).min(SEGMENT - offset);
        let bytes = match segment.read(offset, to_segment_end) {
            Some(bytes) => bytes,
            None => {
                let to_page_end = to_segment_end.min(PAGE - (offset & (PAGE - 1)));
                segment.page_bytes(offset, to_page_end)
            }
        };

        self.pos += bytes.len();
        Some((pos as u64, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound.max(1)
        }
    }

    fn check(segments: &Segments, model: &[u8]) {
        assert_eq!(segments.len(), model.len() as u64);
        assert_eq!(segments.to_vec(), model);
    }

    #[test]
    fn from_bytes_should_hold_them() {
        let bytes: Vec<u8> = (0..SEGMENT * 2 + 100).map(|i| i as u8).collect();
        let segments = Segments::from(&bytes[..]);

        check(&segments, &bytes);
        assert_eq!(
            segments.read(SEGMENT as u64 - 2, 4).to_vec(),
            bytes[SEGMENT - 2..SEGMENT + 2]
        );
    }

    #[test]
    fn read_within_segment_should_borrow() {
        let mut segments = Segments::from(&[1_u8; 4096][..]);
        let snapshot = segments.clone();
        segments.write(10, &[2; 5]);

        assert!(matches!(segments.read(2048, 16), StorageSlice::Borrowed(_)));
        assert!(matches!(segments.read(10, 5), StorageSlice::Borrowed(_)));
        assert!(matches!(segments.read(1020, 8), StorageSlice::Owned(_)));
        assert_eq!(segments.read(8, 4).to_vec(), [1, 1, 2, 2]);
        assert_eq!(snapshot.read(8, 4).to_vec(), [1, 1, 1, 1]);
    }

    #[test]
    fn write_without_clones_should_not_override() {
        let mut segments = Segments::from(&[0_u8; 4096][..]);
        segments.write(100, &[1; 2000]);

        assert!(segments.segment(0).overrides.is_none());
    }

    #[test]
    fn write_after_snapshot_dropped_should_fold() {
        let mut segments = Segments::from(&[0_u8; 4096][..]);
        let snapshot = segments.clone();
        segments.write(0, &[1; 10]);
        assert!(segments.segment(0).overrides.is_some());
        drop(snapshot);

        segments.write(2000, &[2; 10]);

        assert!(segments.segment(0).overrides.is_none());
        assert_eq!(segments.read(0, 10).to_vec(), [1; 10]);
        assert_eq!(segments.read(2000, 10).to_vec(), [2; 10]);
    }

    #[test]
    fn shrink_then_grow_should_zero() {
        let mut segments = Segments::from(&[7_u8; 3000][..]);
        segments.resize(10);
        segments.resize(3000);

        let mut model = vec![0_u8; 3000];
        model[..10].fill(7);
        check(&segments, &model);
    }

    #[test]
    fn write_past_end_should_zero_gap() {
        let mut segments = Segments::from(&[7_u8; 3000][..]);
        segments.resize(100);
        segments.write(2000, &[1; 10]);

        let mut model = vec![0_u8; 2010];
        model[..100].fill(7);
        model[2000..].fill(1);
        check(&segments, &model);
    }

    #[test]
    fn random_writes_and_snapshots_should_match_model() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);

        for round in 0..20 {
            let mut segments = Segments::default();
            let mut model: Vec<u8> = Vec::new();
            let mut snapshots: Vec<(Segments, Vec<u8>)> = Vec::new();
            let max = if round % 2 == 0 {
                3 * SEGMENT as u64
            } else {
                8 * PAGE as u64
            };

            for step in 0..400 {
                match rng.below(10) {
                    0 => {
                        let new_len = rng.below(max);
                        segments.resize(new_len);
                        model.resize(new_len as usize, 0);
                    }
                    1 => snapshots.push((segments.clone(), model.clone())),
                    2 if !snapshots.is_empty() => {
                        let i = rng.below(snapshots.len() as u64) as usize;
                        snapshots.swap_remove(i);
                    }
                    _ => {
                        let pos = rng.below(max);
                        let bound = if rng.below(4) == 0 {
                            3 * PAGE as u64
                        } else {
                            64
                        };
                        let len = rng.below(bound);
                        let value = (step % 251) as u8 + 1;
                        let bytes = vec![value; len as usize];
                        segments.write(pos, &bytes);

                        if !bytes.is_empty() {
                            let end = (pos + len) as usize;
                            if model.len() < end {
                                model.resize(end, 0);
                            }
                            model[pos as usize..end].fill(value);
                        }
                    }
                }

                if !model.is_empty() {
                    let pos = rng.below(model.len() as u64);
                    let len = rng.below(model.len() as u64 - pos + 1);
                    assert_eq!(
                        segments.read(pos, len).to_vec(),
                        model[pos as usize..(pos + len) as usize]
                    );
                }
            }

            check(&segments, &model);

            for (snapshot, expected) in &snapshots {
                check(snapshot, expected);
            }
        }
    }

    #[test]
    fn many_versions_should_merge_and_fold_overrides() {
        let mut segments = Segments::from(&[0_u8; SEGMENT][..]);
        let mut model = vec![0_u8; SEGMENT];
        let mut snapshots = Vec::new();

        for i in 0..2000_usize {
            snapshots.push(segments.clone());
            let pos = (i * 7919) % (SEGMENT - 8);
            segments.write(pos as u64, &[i as u8; 8]);
            model[pos..pos + 8].fill(i as u8);
            let overrides = segments.segment(0).overrides.as_deref();
            assert!(overrides.is_none_or(|o| o.depth <= MAX_DEPTH && o.total <= FOLD_PAGES));
        }

        check(&segments, &model);
    }

    #[test]
    fn chunks_should_cover_range() {
        let bytes: Vec<u8> = (0..SEGMENT + 5000).map(|i| (i % 253) as u8).collect();
        let mut segments = Segments::from(&bytes[..]);
        let _snapshot = segments.clone();
        segments.write(3000, &[0; 3000]);
        let mut model = bytes.clone();
        model[3000..6000].fill(0);

        let mut pos = 10;
        let mut collected = Vec::new();

        for (at, chunk) in segments.chunks(10, SEGMENT as u64 + 4000) {
            assert_eq!(at, pos);
            pos += chunk.len() as u64;
            collected.extend_from_slice(chunk);
        }

        assert_eq!(collected, model[10..SEGMENT + 4000]);
    }
}
