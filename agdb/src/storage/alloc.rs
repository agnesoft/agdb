use super::pages::View;
use super::pages::ViewMut;
use crate::DbError;
use crate::DbErrorType;

/// The bytes of the root at the start of the file, at least one page,
/// never allocated.
pub(crate) const ROOT_BYTES: u64 = 4096;
/// The slab classes of the smallest page, then three more per doubling
/// of the page. A page takes the classes of which two slots fit in it.
const CLASSES: [u64; 30] = [
    16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 168, 200, 248, 336, 504, 672, 840, 1016, 1352, 1688,
    2040, 2720, 3400, 4088, 5448, 6808, 8184, 10912, 13640, 16376,
];
/// The segments of the page map, the `k`-th of `2^k` pages: 20 cover
/// `2^32` pages of the smallest size.
const MAP_SEGMENTS: usize = 20;
const BUCKETS: usize = 32;
/// The bytes of an encoded [`Allocator`].
pub(crate) const ALLOCATOR_SIZE: usize = 4 * (1 + MAP_SEGMENTS + CLASSES.len() + BUCKETS);
const SLAB_HEADER: u64 = 16;
const SLAB_PREV: u64 = 8;
const SLAB_NEXT: u64 = 12;
const RUN_HEADER: u64 = 12;
const RUN_NEXT: u64 = 0;
const RUN_PREV: u64 = 4;
const RUN_LEN: u64 = 8;
/// The runs of its own bucket an allocation looks at before it takes
/// the head of a longer bucket or extends the file.
const RUN_SCAN: u32 = 8;

/// Owns every page past the root: slab pages carve objects of one size
/// class, the larger ones take runs of whole pages. The owner of an
/// allocation keeps its size, so there is no per-allocation header.
///
/// Slab page: `[used u16][free u16][bump u16][pad u16][prev u32][next u32]`
/// then the slots, the free ones threaded by their offsets. The pages
/// with a free slot are listed per class.
///
/// Free run: `[next u32][prev u32][len u32]` in its first page, `len` in
/// the last 4 bytes of its last page, listed by `floor(log2(len))`. The
/// page map holds a bit for the first and the last page of each free run
/// to coalesce runs as they are freed. A free run never touches the end
/// of the file, which is truncated instead.
///
/// The page map grows by segments that never move, the `k`-th covering
/// `2^k` pages of the map, each appended before the pages it was added
/// for. A segment ending the file covers no page, so it is dropped with
/// the free run before it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Allocator {
    shift: u32,
    pages: u32,
    map: [u32; MAP_SEGMENTS],
    slabs: [u32; CLASSES.len()],
    runs: [u32; BUCKETS],
}

struct SlabHeader {
    used: u16,
    free: u16,
    bump: u16,
    prev: u32,
    next: u32,
}

struct RunHeader {
    next: u32,
    prev: u32,
    len: u32,
}

fn u16_at(bytes: &[u8], at: u64) -> u16 {
    let at = at as usize;
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

#[inline]
pub(crate) fn u32_at(bytes: &[u8], at: u64) -> u32 {
    let at = at as usize;
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn read_u16(view: &impl View, pos: u64) -> Result<u16, DbError> {
    Ok(u16_at(&view.read(pos, 2)?, 0))
}

fn read_u32(view: &impl View, pos: u64) -> Result<u32, DbError> {
    Ok(u32_at(&view.read(pos, 4)?, 0))
}

fn write_u32(view: &mut impl ViewMut, pos: u64, value: u32) -> Result<(), DbError> {
    view.write(pos, &value.to_le_bytes())
}

fn bucket(len: u32) -> usize {
    len.ilog2() as usize
}

/// The pages of the root for pages of `1 << shift` bytes.
pub(crate) fn root_pages(shift: u32) -> u32 {
    (ROOT_BYTES >> shift).max(1) as u32
}

impl SlabHeader {
    fn read(view: &impl View, pos: u64) -> Result<Self, DbError> {
        let bytes = view.read(pos, SLAB_HEADER)?;

        Ok(Self {
            used: u16_at(&bytes, 0),
            free: u16_at(&bytes, 2),
            bump: u16_at(&bytes, 4),
            prev: u32_at(&bytes, SLAB_PREV),
            next: u32_at(&bytes, SLAB_NEXT),
        })
    }

    fn write(&self, view: &mut impl ViewMut, pos: u64) -> Result<(), DbError> {
        let mut bytes = [0_u8; SLAB_HEADER as usize];
        bytes[0..2].copy_from_slice(&self.used.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.free.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.bump.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.prev.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.next.to_le_bytes());
        view.write(pos, &bytes)
    }

    fn full(&self, size: u64, page_size: u64) -> bool {
        self.free == 0 && page_size < u64::from(self.bump) + size
    }
}

impl RunHeader {
    fn read(view: &impl View, pos: u64) -> Result<Self, DbError> {
        let bytes = view.read(pos, RUN_HEADER)?;

        Ok(Self {
            next: u32_at(&bytes, RUN_NEXT),
            prev: u32_at(&bytes, RUN_PREV),
            len: u32_at(&bytes, RUN_LEN),
        })
    }

    fn write(&self, view: &mut impl ViewMut, pos: u64) -> Result<(), DbError> {
        let mut bytes = [0_u8; RUN_HEADER as usize];
        bytes[0..4].copy_from_slice(&self.next.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.prev.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.len.to_le_bytes());
        view.write(pos, &bytes)
    }
}

impl Allocator {
    /// The allocator of a file of pages of `1 << shift` bytes holding
    /// only the root.
    pub(crate) fn new(shift: u32) -> Self {
        Self {
            shift,
            pages: root_pages(shift),
            map: [0; MAP_SEGMENTS],
            slabs: [0; CLASSES.len()],
            runs: [0; BUCKETS],
        }
    }

    pub(crate) fn decode(bytes: &[u8; ALLOCATOR_SIZE], shift: u32) -> Self {
        let mut words = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_le_bytes(*word));
        let mut next = || words.next().unwrap_or_default();

        Self {
            shift,
            pages: next(),
            map: std::array::from_fn(|_| next()),
            slabs: std::array::from_fn(|_| next()),
            runs: std::array::from_fn(|_| next()),
        }
    }

    pub(crate) fn encode(&self) -> [u8; ALLOCATOR_SIZE] {
        let mut bytes = [0_u8; ALLOCATOR_SIZE];
        let words = [self.pages]
            .into_iter()
            .chain(self.map)
            .chain(self.slabs)
            .chain(self.runs);

        for (chunk, word) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(words) {
            *chunk = word.to_le_bytes();
        }

        bytes
    }

    /// The length of the file the allocator owns.
    pub(crate) fn len(&self) -> u64 {
        self.page_pos(self.pages)
    }

    pub(crate) fn page_size(&self) -> u64 {
        1 << self.shift
    }

    /// The bytes an allocation of `size` bytes can hold.
    pub(crate) fn usable(&self, size: u64) -> u64 {
        self.class(size)
            .map_or_else(|| self.page_pos(self.run_len(size)), |class| CLASSES[class])
    }

    /// Returns the position of `size` bytes of storage, which may hold
    /// any bytes. The `size` is needed again to free it.
    pub(crate) fn alloc(&mut self, view: &mut impl ViewMut, size: u64) -> Result<u64, DbError> {
        match self.class(size) {
            Some(class) => self.alloc_slot(view, class),
            None => {
                let page = self.alloc_run(view, self.run_len(size))?;
                Ok(self.page_pos(page))
            }
        }
    }

    /// Frees `size` bytes allocated at `pos`.
    pub(crate) fn free(
        &mut self,
        view: &mut impl ViewMut,
        pos: u64,
        size: u64,
    ) -> Result<(), DbError> {
        match self.class(size) {
            Some(class) => self.free_slot(view, class, pos),
            None => self.free_run(view, self.page_of(pos), self.run_len(size)),
        }
    }

    /// Resizes the allocation at `pos` from `size` to `new_size` bytes in
    /// place if it can, otherwise moves its bytes a page at a time.
    /// Returns its position.
    pub(crate) fn realloc(
        &mut self,
        view: &mut impl ViewMut,
        pos: u64,
        size: u64,
        new_size: u64,
    ) -> Result<u64, DbError> {
        match (self.class(size), self.class(new_size)) {
            (Some(class), Some(new_class)) if class == new_class => return Ok(pos),
            (None, None) => {
                let page = self.page_of(pos);
                let len = self.run_len(size);
                let new_len = self.run_len(new_size);

                if new_len < len {
                    self.free_run(view, page + new_len, len - new_len)?;
                }

                if new_len <= len || self.grow_run(view, page, len, new_len)? {
                    return Ok(pos);
                }
            }
            _ => {}
        }

        let new_pos = self.alloc(view, new_size)?;
        let kept = size.min(new_size);
        let mut buffer = Vec::new();
        let mut at = 0;

        while at < kept {
            let len = self.page_size().min(kept - at);
            buffer.clear();
            buffer.extend_from_slice(&view.read(pos + at, len)?);
            view.write(new_pos + at, &buffer)?;
            at += len;
        }

        self.free(view, pos, size)?;
        Ok(new_pos)
    }

    fn page_pos(&self, page: u32) -> u64 {
        u64::from(page) << self.shift
    }

    fn page_of(&self, pos: u64) -> u32 {
        (pos >> self.shift) as u32
    }

    fn tag_pos(&self, last: u32) -> u64 {
        self.page_pos(last) + self.page_size() - 4
    }

    fn class(&self, size: u64) -> Option<usize> {
        CLASSES
            .iter()
            .position(|class| size <= *class)
            .filter(|class| 2 * CLASSES[*class] + SLAB_HEADER <= self.page_size())
    }

    /// Saturates at a length no file can hold so that allocating it fails.
    fn run_len(&self, size: u64) -> u32 {
        size.div_ceil(self.page_size()).min(u64::from(u32::MAX)) as u32
    }

    fn alloc_slot(&mut self, view: &mut impl ViewMut, class: usize) -> Result<u64, DbError> {
        let size = CLASSES[class];
        let mut page = self.slabs[class];
        let mut header = if page == 0 {
            page = self.alloc_run(view, 1)?;
            self.slabs[class] = page;
            SlabHeader {
                used: 0,
                free: 0,
                bump: SLAB_HEADER as u16,
                prev: 0,
                next: 0,
            }
        } else {
            SlabHeader::read(view, self.page_pos(page))?
        };

        let offset = if header.free == 0 {
            header.bump += size as u16;
            header.bump - size as u16
        } else {
            let offset = header.free;
            header.free = read_u16(view, self.page_pos(page) + u64::from(offset))?;
            offset
        };

        header.used += 1;

        if header.full(size, self.page_size()) {
            self.slabs[class] = header.next;
            self.set_slab_prev(view, header.next, 0)?;
            header.next = 0;
        }

        header.write(view, self.page_pos(page))?;
        Ok(self.page_pos(page) + u64::from(offset))
    }

    fn free_slot(
        &mut self,
        view: &mut impl ViewMut,
        class: usize,
        pos: u64,
    ) -> Result<(), DbError> {
        let size = CLASSES[class];
        let page = self.page_of(pos);
        let mut header = SlabHeader::read(view, self.page_pos(page))?;
        let full = header.full(size, self.page_size());
        view.write(pos, &header.free.to_le_bytes())?;
        header.free = (pos - self.page_pos(page)) as u16;
        header.used -= 1;

        if header.used == 0 {
            self.unlink_slab(view, class, &header)?;
            return self.free_run(view, page, 1);
        }

        if full {
            header.next = self.slabs[class];
            self.set_slab_prev(view, header.next, page)?;
            self.slabs[class] = page;
        }

        header.write(view, self.page_pos(page))
    }

    fn set_slab_prev(&self, view: &mut impl ViewMut, page: u32, prev: u32) -> Result<(), DbError> {
        if page == 0 {
            return Ok(());
        }

        write_u32(view, self.page_pos(page) + SLAB_PREV, prev)
    }

    fn unlink_slab(
        &mut self,
        view: &mut impl ViewMut,
        class: usize,
        header: &SlabHeader,
    ) -> Result<(), DbError> {
        if header.prev == 0 {
            self.slabs[class] = header.next;
        } else {
            write_u32(view, self.page_pos(header.prev) + SLAB_NEXT, header.next)?;
        }

        self.set_slab_prev(view, header.next, header.prev)
    }

    fn alloc_run(&mut self, view: &mut impl ViewMut, len: u32) -> Result<u32, DbError> {
        let bucket = bucket(len);
        let mut page = self.runs[bucket];

        for _ in 0..RUN_SCAN {
            if page == 0 {
                break;
            }

            let header = RunHeader::read(view, self.page_pos(page))?;

            if len <= header.len {
                return self.take_run(view, page, &header, len);
            }

            page = header.next;
        }

        if let Some(page) = self.runs[bucket + 1..]
            .iter()
            .copied()
            .find(|page| *page != 0)
        {
            let header = RunHeader::read(view, self.page_pos(page))?;
            return self.take_run(view, page, &header, len);
        }

        self.extend(view, len)
    }

    fn take_run(
        &mut self,
        view: &mut impl ViewMut,
        page: u32,
        header: &RunHeader,
        len: u32,
    ) -> Result<u32, DbError> {
        self.unlink_run(view, page, header)?;

        if len < header.len {
            self.insert_run(view, page + len, header.len - len)?;
        }

        Ok(page)
    }

    fn grow_run(
        &mut self,
        view: &mut impl ViewMut,
        page: u32,
        len: u32,
        new_len: u32,
    ) -> Result<bool, DbError> {
        let end = page + len;

        if end == self.pages {
            let new_end = u64::from(page) + u64::from(new_len);
            let covered = self.covered(self.map_segments()).min(u64::from(u32::MAX));

            if covered < new_end {
                return Ok(false);
            }

            self.pages = new_end as u32;
            view.set_len(self.page_pos(self.pages));
            return Ok(true);
        }

        if !self.is_free(view, end)? {
            return Ok(false);
        }

        let next = RunHeader::read(view, self.page_pos(end))?;

        if len + next.len < new_len {
            return Ok(false);
        }

        self.unlink_run(view, end, &next)?;

        if new_len < len + next.len {
            self.insert_run(view, page + new_len, len + next.len - new_len)?;
        }

        Ok(true)
    }

    fn free_run(
        &mut self,
        view: &mut impl ViewMut,
        mut page: u32,
        mut len: u32,
    ) -> Result<(), DbError> {
        if self.is_free(view, page - 1)? {
            let first = page - read_u32(view, self.tag_pos(page - 1))?;
            let prev = RunHeader::read(view, self.page_pos(first))?;
            self.unlink_run(view, first, &prev)?;
            page = first;
            len += prev.len;
        }

        if page + len == self.pages {
            return self.truncate(view, page);
        }

        if self.is_free(view, page + len)? {
            let next = RunHeader::read(view, self.page_pos(page + len))?;
            self.unlink_run(view, page + len, &next)?;
            len += next.len;
        }

        self.insert_run(view, page, len)
    }

    fn insert_run(&mut self, view: &mut impl ViewMut, page: u32, len: u32) -> Result<(), DbError> {
        let next = self.runs[bucket(len)];

        if next != 0 {
            write_u32(view, self.page_pos(next) + RUN_PREV, page)?;
        }

        RunHeader { next, prev: 0, len }.write(view, self.page_pos(page))?;
        write_u32(view, self.tag_pos(page + len - 1), len)?;
        self.runs[bucket(len)] = page;
        self.set_free(view, page, true)?;
        self.set_free(view, page + len - 1, true)
    }

    fn unlink_run(
        &mut self,
        view: &mut impl ViewMut,
        page: u32,
        header: &RunHeader,
    ) -> Result<(), DbError> {
        if header.prev == 0 {
            self.runs[bucket(header.len)] = header.next;
        } else {
            write_u32(view, self.page_pos(header.prev) + RUN_NEXT, header.next)?;
        }

        if header.next != 0 {
            write_u32(view, self.page_pos(header.next) + RUN_PREV, header.prev)?;
        }

        self.set_free(view, page, false)?;
        self.set_free(view, page + header.len - 1, false)
    }

    /// Appends the segments of the page map it needs to cover `len` more
    /// pages, then the pages. Returns the first page.
    fn extend(&mut self, view: &mut impl ViewMut, len: u32) -> Result<u32, DbError> {
        let mut page = u64::from(self.pages);
        let mut map = self.map;
        let mut segments = self.map_segments();

        while self.covered(segments) < page + u64::from(len)
            && page + u64::from(len) <= u64::from(u32::MAX)
        {
            map[segments] = page as u32;
            page += 1 << segments;
            segments += 1;
        }

        let end = page + u64::from(len);

        if u64::from(u32::MAX) < end {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!("Storage full ({end} pages)"),
            ));
        }

        self.pages = end as u32;
        self.map = map;
        view.set_len(self.page_pos(self.pages));
        Ok(page as u32)
    }

    /// Truncates the file to `page`, then drops each segment of the page
    /// map that ends it with the free run before it.
    fn truncate(&mut self, view: &mut impl ViewMut, mut page: u32) -> Result<(), DbError> {
        while let Some(segment) = self.map_segments().checked_sub(1)
            && self.map[segment] + (1 << segment) == page
        {
            page = self.map[segment];

            if self.is_free(view, page - 1)? {
                let first = page - read_u32(view, self.tag_pos(page - 1))?;
                let run = RunHeader::read(view, self.page_pos(first))?;
                self.unlink_run(view, first, &run)?;
                page = first;
            }

            self.map[segment] = 0;
        }

        self.pages = page;
        view.set_len(self.page_pos(page));
        Ok(())
    }

    fn map_segments(&self) -> usize {
        self.map.iter().take_while(|page| **page != 0).count()
    }

    /// The pages the first `segments` of the map cover.
    fn covered(&self, segments: usize) -> u64 {
        ((1 << segments) - 1) << (self.shift + 3)
    }

    fn map_pos(&self, page: u32) -> u64 {
        let byte = u64::from(page / 8);
        let map_page = (byte >> self.shift) + 1;
        let segment = map_page.ilog2();
        self.page_pos(self.map[segment as usize])
            + ((map_page - (1 << segment)) << self.shift)
            + (byte & (self.page_size() - 1))
    }

    fn is_free(&self, view: &impl View, page: u32) -> Result<bool, DbError> {
        Ok(view.read(self.map_pos(page), 1)?[0] & (1 << (page % 8)) != 0)
    }

    fn set_free(&self, view: &mut impl ViewMut, page: u32, free: bool) -> Result<(), DbError> {
        let pos = self.map_pos(page);
        let byte = view.read(pos, 1)?[0];
        let bit = 1 << (page % 8);
        view.write(pos, &[if free { byte | bit } else { byte & !bit }])
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::DbErrorType;
    use crate::storage::StorageData;
    use crate::storage::StorageSlice;
    use crate::storage::file_storage::FileStorageImpl;
    use crate::storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
    use crate::storage::memory_storage::MemoryStorageImpl;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::MAX_PAGE_SHIFT;
    use crate::storage::pages::Overlay;
    use crate::storage::pages::Reader;
    use crate::storage::pages::Writer;
    use crate::storage::pages::tests::XorShift;
    use crate::storage::storage_simulator::tests::SCALE;
    use crate::storage::storage_simulator::tests::Scale;
    use crate::storage::storage_simulator::tests::Simulation;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_platform;
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    pub(crate) type Heap = MemoryStorageImpl<TestPlatform>;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Owner {
        None,
        Root,
        Map,
        Free,
        Slab(usize),
        Run,
    }

    /// Fails its `fail`-th read or write.
    pub(crate) struct Faulty<'a, V> {
        view: &'a mut V,
        pub(crate) calls: Cell<u64>,
        pub(crate) fail: u64,
    }

    impl<'a, V> Faulty<'a, V> {
        pub(crate) fn new(view: &'a mut V, fail: u64) -> Self {
            Self {
                view,
                calls: Cell::new(0),
                fail,
            }
        }

        fn tick(&self) -> Result<(), DbError> {
            let call = self.calls.get();
            self.calls.set(call + 1);

            if call == self.fail {
                return Err(DbError::storage(DbErrorType::NotAllowed, "fault"));
            }

            Ok(())
        }
    }

    impl<V: View> View for Faulty<'_, V> {
        fn len(&self) -> u64 {
            self.view.len()
        }

        fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
            self.tick()?;
            self.view.read(pos, len)
        }
    }

    impl<V: ViewMut> ViewMut for Faulty<'_, V> {
        fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
            self.tick()?;
            self.view.write(pos, bytes)
        }

        fn set_len(&mut self, len: u64) {
            self.view.set_len(len);
        }
    }

    pub(crate) fn heap() -> Heap {
        heap_with(DEFAULT_PAGE_SHIFT)
    }

    /// A heap holding the root of pages of `1 << shift` bytes.
    pub(crate) fn heap_with(shift: u32) -> Heap {
        MemoryStorageImpl::from_buffer_with_platform(
            Arc::new(test_platform()),
            "/heap",
            vec![0; Allocator::new(shift).len() as usize],
        )
    }

    pub(crate) fn pattern(id: u64, size: u64) -> Vec<u8> {
        (0..size).map(|i| (id * 31 + i) as u8).collect()
    }

    fn bytes(view: &impl View, pos: u64, size: u64) -> Vec<u8> {
        view.read(pos, size).unwrap().to_vec()
    }

    /// Asserts the invariants of the allocator given its `live`
    /// allocations (pos, size): every page is owned exactly once, the
    /// lists are linked both ways, the slab pages count their slots, the
    /// free runs are coalesced and marked in the page map.
    pub(crate) fn verify(allocator: &Allocator, view: &impl View, live: &[(u64, u64)]) {
        let page_size = allocator.page_size();
        assert_eq!(view.len(), allocator.len());
        let mut owners = vec![Owner::None; allocator.pages as usize];
        let own = |owners: &mut Vec<Owner>, first: u32, len: u32, owner: Owner| {
            for page in first..first + len {
                assert_eq!(owners[page as usize], Owner::None, "page {page}");
                owners[page as usize] = owner;
            }
        };
        own(&mut owners, 0, root_pages(allocator.shift), Owner::Root);

        let segments = allocator.map_segments();

        for (segment, page) in allocator.map[..segments].iter().enumerate() {
            own(&mut owners, *page, 1 << segment, Owner::Map);
            let before = allocator
                .covered(segment)
                .max(root_pages(allocator.shift).into());
            assert!(
                u64::from(*page) <= before,
                "segment {segment} after its pages"
            );
        }

        if let Some(top) = segments.checked_sub(1) {
            let end = allocator.map[top] + (1 << top);
            assert_ne!(end, allocator.pages, "segment {top} not dropped");
        }

        assert!(allocator.map[segments..].iter().all(|page| *page == 0));

        let mut runs = Vec::new();

        for (bucket_index, head) in allocator.runs.iter().enumerate() {
            let mut prev = 0;
            let mut page = *head;

            while page != 0 {
                let header = RunHeader::read(view, allocator.page_pos(page)).unwrap();
                assert_eq!(header.prev, prev, "run {page}");
                assert_eq!(bucket(header.len), bucket_index, "run {page}");
                assert_eq!(
                    read_u32(view, allocator.tag_pos(page + header.len - 1)).unwrap(),
                    header.len
                );
                own(&mut owners, page, header.len, Owner::Free);
                runs.push((page, header.len));
                prev = page;
                page = header.next;
            }
        }

        let mut listed = BTreeSet::new();

        for (class_index, head) in allocator.slabs.iter().enumerate() {
            let mut prev = 0;
            let mut page = *head;

            while page != 0 {
                let header = SlabHeader::read(view, allocator.page_pos(page)).unwrap();
                assert_eq!(header.prev, prev, "slab {page}");
                own(&mut owners, page, 1, Owner::Slab(class_index));
                listed.insert(page);
                prev = page;
                page = header.next;
            }
        }

        let mut slots: BTreeMap<u32, BTreeSet<u64>> = BTreeMap::new();

        for (pos, size) in live.iter().copied() {
            let page = allocator.page_of(pos);

            if let Some(class_index) = allocator.class(size) {
                if owners[page as usize] == Owner::None {
                    owners[page as usize] = Owner::Slab(class_index);
                }

                assert_eq!(owners[page as usize], Owner::Slab(class_index), "{pos}");
                let offset = pos - allocator.page_pos(page);
                assert_eq!((offset - SLAB_HEADER) % CLASSES[class_index], 0, "{pos}");
                assert!(slots.entry(page).or_default().insert(offset), "{pos}");
            } else {
                assert_eq!(pos % page_size, 0, "{pos}");
                own(&mut owners, page, allocator.run_len(size), Owner::Run);
            }
        }

        for (page, owner) in owners.iter().enumerate() {
            let page = page as u32;
            assert_ne!(*owner, Owner::None, "page {page} leaked");

            if let Owner::Slab(class_index) = owner {
                let size = CLASSES[*class_index];
                let header = SlabHeader::read(view, allocator.page_pos(page)).unwrap();
                let used = slots.remove(&page).unwrap_or_default();
                assert_eq!(usize::from(header.used), used.len(), "slab {page}");
                assert_ne!(header.used, 0, "slab {page} not reclaimed");
                let mut free = BTreeSet::new();
                let mut offset = u64::from(header.free);

                while offset != 0 {
                    assert_eq!((offset - SLAB_HEADER) % size, 0, "slab {page}");
                    assert!(offset < u64::from(header.bump), "slab {page}");
                    assert!(!used.contains(&offset), "slab {page}");
                    assert!(free.insert(offset), "slab {page}");
                    offset = u64::from(read_u16(view, allocator.page_pos(page) + offset).unwrap());
                }

                assert_eq!(
                    (used.len() + free.len()) as u64,
                    (u64::from(header.bump) - SLAB_HEADER) / size
                );
                assert_eq!(
                    listed.contains(&page),
                    !header.full(size, page_size),
                    "slab {page}"
                );
            }
        }

        let mut bounds = BTreeSet::new();

        for (page, len) in runs {
            assert_ne!(owners[page as usize - 1], Owner::Free, "run {page}");
            assert!(page + len < allocator.pages, "run {page} not truncated");
            assert_ne!(owners[(page + len) as usize], Owner::Free, "run {page}");
            bounds.insert(page);
            bounds.insert(page + len - 1);
        }

        if allocator.map_segments() != 0 {
            for page in 0..allocator.pages {
                assert_eq!(
                    allocator.is_free(view, page).unwrap(),
                    bounds.contains(&page),
                    "map bit {page}"
                );
            }
        }
    }

    /// Mostly slab sizes, some runs of a few pages and a few long ones.
    fn random_size(random: &mut XorShift, shift: u32) -> u64 {
        let scale = 1 << (shift - DEFAULT_PAGE_SHIFT);
        let half = 512 * scale;

        match random.next(10) {
            0..=6 => 1 + random.next(half - 8),
            7 | 8 => half - 7 + random.next(5000 * scale),
            _ => 5000 * scale + random.next(40000 * scale),
        }
    }

    /// Allocates, frees or reallocates at random checking the bytes of
    /// the touched allocations, `live` being (pos, size, pattern id).
    pub(crate) fn random_op(
        allocator: &mut Allocator,
        view: &mut impl ViewMut,
        live: &mut Vec<(u64, u64, u64)>,
        random: &mut XorShift,
        id: u64,
    ) -> Result<(), DbError> {
        let op = if live.is_empty() { 0 } else { random.next(10) };

        if op < 5 {
            let size = random_size(random, allocator.shift);
            let pos = allocator.alloc(view, size)?;
            view.write(pos, &pattern(id, size))?;
            live.push((pos, size, id));
            return Ok(());
        }

        let index = random.next(live.len() as u64) as usize;
        let (pos, size, old_id) = live[index];
        assert_eq!(view.read(pos, size)?.to_vec(), pattern(old_id, size));

        if op < 8 {
            allocator.free(view, pos, size)?;
            live.swap_remove(index);
        } else {
            let new_size = random_size(random, allocator.shift);
            let new_pos = allocator.realloc(view, pos, size, new_size)?;
            let kept = size.min(new_size);
            assert_eq!(
                view.read(new_pos, kept)?.to_vec(),
                pattern(old_id, size)[..kept as usize]
            );
            view.write(new_pos, &pattern(id, new_size))?;
            live[index] = (new_pos, new_size, id);
        }

        Ok(())
    }

    /// Grows the run at 6 into part of the free run after it and then
    /// into the rest, moves it past a new segment of the page map a page
    /// at a time, then frees all, which drops the segments.
    fn scripted(allocator: &mut Allocator, view: &mut impl ViewMut) -> Result<(), DbError> {
        allocator.free(view, 7 * 1024, 3072)?;
        allocator.realloc(view, 6 * 1024, 1024, 2048)?;
        allocator.realloc(view, 6 * 1024, 2048, 4096)?;
        let moved = allocator.realloc(view, 6 * 1024, 4096, 9 * 1024 * 1024)?;
        allocator.free(view, 10 * 1024, 1024)?;
        allocator.free(view, moved, 9 * 1024 * 1024)?;
        allocator.free(view, 5 * 1024 + 16, 16)
    }

    pub(crate) fn sizes(live: &[(u64, u64, u64)]) -> Vec<(u64, u64)> {
        live.iter().map(|(pos, size, _)| (*pos, *size)).collect()
    }

    fn with_root<S: StorageData>(mut storage: S, limit: u64) -> S {
        storage.write(0, &vec![0; ROOT_BYTES as usize]).unwrap();
        storage.flush().unwrap();
        storage.set_overlay_limit(limit);
        storage
    }

    /// Runs ten transactions of random operations over one overlay,
    /// rolling back about a third of them, and verifies the allocator
    /// after each operation and transaction.
    fn random_transactions<S: StorageData>(storage: &mut S, seed: u64, shift: u32) {
        let mut random = XorShift(seed);
        let mut overlay = Overlay::new(shift);
        let mut allocator = Allocator::new(shift);
        let mut live = Vec::new();
        let mut id = 0;

        for round in 0..10 {
            let mut writer = Writer::new(&mut *storage, &mut overlay).unwrap();
            let mut changed = (allocator.clone(), live.clone());

            let mut view = Faulty::new(&mut writer, u64::MAX);

            for _ in 0..100 {
                id += 1;
                random_op(&mut changed.0, &mut view, &mut changed.1, &mut random, id).unwrap();
                verify(&changed.0, &view, &sizes(&changed.1));
            }

            if random.next(3) == 0 {
                writer.rollback().unwrap();
            } else {
                writer.commit().unwrap();
                (allocator, live) = changed;
            }

            let reader = Reader::new(&*storage);
            verify(&allocator, &reader, &sizes(&live));

            for (pos, size, id) in live.iter().copied() {
                assert_eq!(
                    bytes(&reader, pos, size),
                    pattern(id, size),
                    "seed: {seed}, round: {round}"
                );
            }
        }
    }

    mod root_pages {
        use super::*;

        #[test]
        fn root_pages_should_hold_root_in_at_least_one_page() {
            assert_eq!(root_pages(DEFAULT_PAGE_SHIFT), 4);
            assert_eq!(root_pages(12), 1);
            assert_eq!(root_pages(MAX_PAGE_SHIFT), 1);
        }
    }

    mod allocator {
        use super::*;

        #[test]
        fn allocator_should_keep_allocations_over_random_transactions() {
            for seed in 1..=10 {
                random_transactions(&mut heap(), seed, DEFAULT_PAGE_SHIFT);
            }
        }

        #[test]
        fn allocator_with_largest_pages_should_keep_allocations_over_random_transactions() {
            for seed in 1..=5 {
                random_transactions(&mut heap_with(MAX_PAGE_SHIFT), seed, MAX_PAGE_SHIFT);
            }
        }

        #[test]
        fn allocator_should_keep_allocations_over_random_spilled_transactions() {
            let platform = Arc::new(test_platform());

            for seed in 1..=3 {
                for limit in [0, 4096, 65536] {
                    let name = format!("/file{seed}_{limit}");
                    let file = FileStorageImpl::with_platform(platform.clone(), &name).unwrap();
                    random_transactions(&mut with_root(file, limit), seed, DEFAULT_PAGE_SHIFT);
                    let name = format!("/mapped{seed}_{limit}");
                    let mapped =
                        FileStorageMemoryMappedImpl::with_platform(platform.clone(), &name)
                            .unwrap();
                    random_transactions(&mut with_root(mapped, limit), seed, DEFAULT_PAGE_SHIFT);
                }
            }
        }

        #[test]
        fn allocator_should_do_flat_io_from_1k_to_100k_allocations() {
            let mut simulation = Simulation::new();
            let mut random = XorShift(1);
            let mut live = Vec::new();
            let mut allocs = Scale::default();
            let mut frees = Scale::default();

            while (live.len() as u64) < SCALE {
                if random.next(4) == 0 && !live.is_empty() {
                    let (pos, size) = live.swap_remove(random.next(live.len() as u64) as usize);
                    simulation.run(&mut frees, live.len() as u64 + 1, |view, alloc| {
                        alloc.free(view, pos, size)
                    });
                } else {
                    let size = if random.next(128) == 0 {
                        1024 * (1 + random.next(4))
                    } else {
                        1 + random.next(64)
                    };
                    let size_before = live.len() as u64;
                    let pos = simulation.run(&mut allocs, size_before, |view, alloc| {
                        alloc.alloc(view, size)
                    });
                    live.push((pos, size));
                }
            }

            while !live.is_empty() {
                let (pos, size) = live.swap_remove(random.next(live.len() as u64) as usize);
                simulation.run(&mut frees, live.len() as u64 + 1, |view, alloc| {
                    alloc.free(view, pos, size)
                });
            }

            allocs.assert_flat();
            frees.assert_flat();
            verify(&simulation.alloc, &Reader::new(&simulation.storage), &[]);
        }

        #[test]
        fn allocator_failure_should_fail() {
            for fail in 0.. {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                for size in [16, 1024, 3072, 1024] {
                    allocator.alloc(&mut writer, size).unwrap();
                }

                let mut view = Faulty::new(&mut writer, fail);
                let mut random = XorShift(7);
                let mut live = Vec::new();

                let result = scripted(&mut allocator, &mut view).and_then(|()| {
                    (1..=150).try_for_each(|id| {
                        random_op(&mut allocator, &mut view, &mut live, &mut random, id)
                    })
                });

                if view.calls.get() <= fail {
                    result.unwrap();
                    view.fail = u64::MAX;
                    assert!(allocator.alloc(&mut view, u64::MAX).is_err());
                    verify(&allocator, &view, &sizes(&live));
                    break;
                }

                assert!(result.is_err(), "fail: {fail}");
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_hold_only_root() {
                let allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                assert_eq!(allocator.len(), ROOT_BYTES);
                assert_eq!(allocator.map_segments(), 0);
            }
        }

        mod usable {
            use super::*;

            #[test]
            fn usable_should_round_up_to_class_or_pages() {
                let allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                assert_eq!(allocator.usable(1), 16);
                assert_eq!(allocator.usable(40), 48);
                assert_eq!(allocator.usable(504), 504);
                assert_eq!(allocator.usable(505), 1024);
                assert_eq!(allocator.usable(1025), 2048);
            }

            #[test]
            fn usable_with_larger_pages_should_take_classes_two_slots_fit() {
                let allocator = Allocator::new(MAX_PAGE_SHIFT);

                assert_eq!(allocator.usable(505), 672);
                assert_eq!(allocator.usable(16376), 16376);
                assert_eq!(allocator.usable(16377), 32768);
                assert_eq!(Allocator::new(11).usable(1017), 2048);
            }
        }

        mod alloc {
            use super::*;

            #[test]
            fn alloc_should_round_small_sizes_up_to_class() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                let positions: Vec<u64> = [1, 16, 17, 504]
                    .into_iter()
                    .map(|size| allocator.alloc(&mut writer, size).unwrap())
                    .collect();

                assert_eq!(positions, [5136, 5152, 6160, 7184]);
                verify(
                    &allocator,
                    &writer,
                    &[(5136, 1), (5152, 16), (6160, 17), (7184, 504)],
                );
            }

            #[test]
            fn alloc_should_fill_slab_page_before_taking_another() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                let live: Vec<(u64, u64)> = (0..64)
                    .map(|_| (allocator.alloc(&mut writer, 16).unwrap(), 16))
                    .collect();

                let expected: Vec<(u64, u64)> = (0..63)
                    .map(|slot| (5 * 1024 + 16 + slot * 16, 16))
                    .chain([(6 * 1024 + 16, 16)])
                    .collect();
                assert_eq!(live, expected);
                assert_eq!(allocator.slabs[0], 6);
                verify(&allocator, &writer, &live);
            }

            #[test]
            fn alloc_large_should_take_whole_pages() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                let first = allocator.alloc(&mut writer, 505).unwrap();
                let second = allocator.alloc(&mut writer, 2049).unwrap();

                assert_eq!((first, second), (5120, 6144));
                assert_eq!(allocator.pages, 9);
                verify(&allocator, &writer, &[(first, 505), (second, 2049)]);
            }

            #[test]
            fn alloc_large_should_take_first_free_run_long_enough() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let short = allocator.alloc(&mut writer, 2048).unwrap();
                let first = allocator.alloc(&mut writer, 1024).unwrap();
                let long = allocator.alloc(&mut writer, 3072).unwrap();
                let second = allocator.alloc(&mut writer, 1024).unwrap();
                allocator.free(&mut writer, long, 3072).unwrap();
                allocator.free(&mut writer, short, 2048).unwrap();

                let pos = allocator.alloc(&mut writer, 3072).unwrap();

                assert_eq!(pos, long);
                verify(
                    &allocator,
                    &writer,
                    &[(first, 1024), (pos, 3072), (second, 1024)],
                );
            }

            #[test]
            fn alloc_large_should_split_free_run_of_longer_bucket() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let long = allocator.alloc(&mut writer, 8 * 1024).unwrap();
                let other = allocator.alloc(&mut writer, 1024).unwrap();
                allocator.free(&mut writer, long, 8 * 1024).unwrap();

                let pos = allocator.alloc(&mut writer, 3 * 1024).unwrap();

                assert_eq!(pos, long);
                assert_eq!(allocator.runs[2], 8);
                verify(&allocator, &writer, &[(pos, 3 * 1024), (other, 1024)]);
            }

            #[test]
            fn alloc_large_should_take_longer_bucket_past_scanned_runs() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let mut live = Vec::new();
                let mut free = Vec::new();

                for size in [7 * 1024, 16 * 1024]
                    .into_iter()
                    .chain([4 * 1024; RUN_SCAN as usize])
                {
                    free.push((allocator.alloc(&mut writer, size).unwrap(), size));
                    live.push((allocator.alloc(&mut writer, 1024).unwrap(), 1024));
                }

                for (pos, size) in free.iter().copied() {
                    allocator.free(&mut writer, pos, size).unwrap();
                }

                let pos = allocator.alloc(&mut writer, 7 * 1024).unwrap();

                assert_eq!(pos, free[1].0);
                live.push((pos, 7 * 1024));
                verify(&allocator, &writer, &live);
            }

            #[test]
            fn alloc_past_page_map_should_append_segment_before_pages() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();

                let large = allocator.alloc(&mut writer, 9 * 1024 * 1024).unwrap();

                assert_eq!(allocator.map[..3], [4, 6, 0]);
                assert_eq!(large, 8 * 1024);
                assert_eq!(allocator.runs, [0; BUCKETS]);
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (large, 9 * 1024 * 1024)],
                );
            }

            #[test]
            fn alloc_past_storage_capacity_should_fail() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);

                let error = allocator.alloc(&mut writer, u64::MAX).unwrap_err();

                assert_eq!(error.description, "Storage full (4294967299 pages)");
                assert_eq!(allocator, Allocator::new(DEFAULT_PAGE_SHIFT));
                assert_eq!(writer.len(), ROOT_BYTES);
            }

            #[test]
            fn alloc_past_storage_capacity_with_page_map_should_fail() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator {
                    shift: DEFAULT_PAGE_SHIFT,
                    pages: (1 << 31) + 1,
                    map: std::array::from_fn(|segment| u32::from(segment < 19)),
                    slabs: [0; CLASSES.len()],
                    runs: [0; BUCKETS],
                };
                let before = allocator.clone();
                let size = allocator.page_pos(u32::MAX - allocator.pages - 10);

                let error = allocator.alloc(&mut writer, size).unwrap_err();

                assert_eq!(error.description, "Storage full (4295491573 pages)");
                assert_eq!(allocator, before);
            }
        }

        mod free {
            use super::*;

            #[test]
            fn free_should_reclaim_empty_slab_page() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let pos = allocator.alloc(&mut writer, 16).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();

                allocator.free(&mut writer, pos, 16).unwrap();

                assert_eq!(allocator.slabs[0], 0);
                assert_eq!(allocator.runs[0], 5);
                verify(&allocator, &writer, &[(last, 1024)]);
            }

            #[test]
            fn free_slot_of_full_slab_page_should_list_page_again() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let mut live: Vec<(u64, u64)> = (0..63)
                    .map(|_| (allocator.alloc(&mut writer, 16).unwrap(), 16))
                    .collect();
                let (pos, _) = live.remove(1);

                allocator.free(&mut writer, pos, 16).unwrap();

                assert_eq!(allocator.slabs[0], 5);
                verify(&allocator, &writer, &live);
                assert_eq!(allocator.alloc(&mut writer, 16).unwrap(), pos);
            }

            #[test]
            fn free_last_slot_of_listed_slab_page_should_unlink_it() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let mut live: Vec<(u64, u64)> = (0..64)
                    .map(|_| (allocator.alloc(&mut writer, 16).unwrap(), 16))
                    .collect();
                let (first, _) = live.remove(0);
                let (last, _) = live.pop().unwrap();
                allocator.free(&mut writer, first, 16).unwrap();

                allocator.free(&mut writer, last, 16).unwrap();

                assert_eq!(allocator.slabs[0], 5);
                assert_eq!(allocator.pages, 6);
                verify(&allocator, &writer, &live);
            }

            #[test]
            fn free_should_coalesce_with_free_neighbours() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let runs: Vec<u64> = (0..3)
                    .map(|_| allocator.alloc(&mut writer, 2048).unwrap())
                    .collect();
                let last = allocator.alloc(&mut writer, 1024).unwrap();
                allocator.free(&mut writer, runs[0], 2048).unwrap();
                allocator.free(&mut writer, runs[2], 2048).unwrap();

                allocator.free(&mut writer, runs[1], 2048).unwrap();

                assert_eq!(allocator.runs[2], 6);
                assert_eq!(RunHeader::read(&writer, 6 * 1024).unwrap().len, 6);
                verify(&allocator, &writer, &[(small, 16), (last, 1024)]);
            }

            #[test]
            fn free_at_end_should_truncate_file() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let before = allocator.alloc(&mut writer, 1024).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();
                allocator.free(&mut writer, before, 1024).unwrap();

                allocator.free(&mut writer, last, 1024).unwrap();

                assert_eq!(allocator.pages, 6);
                assert_eq!(writer.len(), 6 * 1024);
                assert_eq!(allocator.runs, [0; BUCKETS]);
                verify(&allocator, &writer, &[(small, 16)]);
            }

            #[test]
            fn free_at_end_should_drop_segments_of_page_map() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let large = allocator.alloc(&mut writer, 64 * 1024 * 1024).unwrap();
                assert_eq!(allocator.map[..5], [4, 6, 8, 12, 0]);

                allocator
                    .free(&mut writer, large, 64 * 1024 * 1024)
                    .unwrap();
                assert_eq!(allocator.map[..2], [4, 0]);
                assert_eq!(writer.len(), 6 * 1024);
                verify(&allocator, &writer, &[(small, 16)]);
                allocator.free(&mut writer, small, 16).unwrap();

                assert_eq!(allocator, Allocator::new(DEFAULT_PAGE_SHIFT));
                assert_eq!(writer.len(), ROOT_BYTES);
            }

            #[test]
            fn free_at_end_should_drop_segment_with_free_run_before_it() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let before = allocator.alloc(&mut writer, 1024).unwrap();
                let large = allocator.alloc(&mut writer, 9 * 1024 * 1024).unwrap();
                allocator.free(&mut writer, before, 1024).unwrap();

                allocator.free(&mut writer, large, 9 * 1024 * 1024).unwrap();

                assert_eq!(allocator.map[..2], [4, 0]);
                assert_eq!(allocator.runs, [0; BUCKETS]);
                assert_eq!(writer.len(), 6 * 1024);
                verify(&allocator, &writer, &[(small, 16)]);
            }
        }

        mod realloc {
            use super::*;

            #[test]
            fn realloc_within_class_should_keep_position() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let pos = allocator.alloc(&mut writer, 10).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 10, 16).unwrap();

                assert_eq!(new_pos, pos);
                verify(&allocator, &writer, &[(pos, 16)]);
            }

            #[test]
            fn realloc_shorter_run_should_free_tail_pages() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 4096).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 4096, 1500).unwrap();

                assert_eq!(new_pos, pos);
                assert_eq!(allocator.runs[1], 8);
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (pos, 1500), (last, 1024)],
                );
            }

            #[test]
            fn realloc_run_at_end_should_grow_in_place() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 1024).unwrap();
                writer.write(pos, &pattern(1, 1024)).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 1024, 3000).unwrap();

                assert_eq!(new_pos, pos);
                assert_eq!(allocator.pages, 9);
                assert_eq!(bytes(&writer, pos, 1024), pattern(1, 1024));
                verify(&allocator, &writer, &[(small, 16), (pos, 3000)]);
            }

            #[test]
            fn realloc_run_should_grow_into_free_run_after_it() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 1024).unwrap();
                let next = allocator.alloc(&mut writer, 3072).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();
                allocator.free(&mut writer, next, 3072).unwrap();

                let split = allocator.realloc(&mut writer, pos, 1024, 2048).unwrap();
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (pos, 2048), (last, 1024)],
                );
                let whole = allocator.realloc(&mut writer, pos, 2048, 4096).unwrap();

                assert_eq!((split, whole), (pos, pos));
                assert_eq!(allocator.runs, [0; BUCKETS]);
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (pos, 4096), (last, 1024)],
                );
            }

            #[test]
            fn realloc_run_at_end_past_page_map_should_move_past_new_segment() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 1024).unwrap();
                writer.write(pos, &pattern(1, 1024)).unwrap();

                let new_pos = allocator
                    .realloc(&mut writer, pos, 1024, 9 * 1024 * 1024)
                    .unwrap();

                assert_eq!(new_pos, 9 * 1024);
                assert_eq!(allocator.map[..3], [4, 7, 0]);
                assert_eq!(allocator.runs[0], 6);
                assert_eq!(bytes(&writer, new_pos, 1024), pattern(1, 1024));
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (new_pos, 9 * 1024 * 1024)],
                );
            }

            #[test]
            fn realloc_run_at_end_past_storage_capacity_should_fail() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let mut view = Faulty::new(&mut writer, u64::MAX);
                allocator.alloc(&mut view, 1024).unwrap();
                let pos = allocator.alloc(&mut view, 1024).unwrap();
                let before = allocator.clone();

                let error = allocator
                    .realloc(&mut view, pos, 1024, u64::MAX)
                    .unwrap_err();

                assert_eq!(pos, allocator.len() - 1024);
                assert_eq!(error.description, "Storage full (4294967302 pages)");
                assert_eq!(allocator, before);
            }

            #[test]
            fn realloc_run_should_move_bytes_a_page_at_a_time() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let pos = allocator.alloc(&mut writer, 2500).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();
                writer.write(pos, &pattern(1, 2500)).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 2500, 5000).unwrap();

                assert_ne!(new_pos, pos);
                assert_eq!(bytes(&writer, new_pos, 2500), pattern(1, 2500));
                verify(&allocator, &writer, &[(last, 1024), (new_pos, 5000)]);
            }

            #[test]
            fn realloc_run_before_short_free_run_should_move() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 1024).unwrap();
                let next = allocator.alloc(&mut writer, 1024).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();
                writer.write(pos, &pattern(1, 1024)).unwrap();
                allocator.free(&mut writer, next, 1024).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 1024, 4096).unwrap();

                assert_eq!(new_pos, 9 * 1024);
                assert_eq!(bytes(&writer, new_pos, 1024), pattern(1, 1024));
                assert_eq!(allocator.runs[1], 6);
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (last, 1024), (new_pos, 4096)],
                );
            }

            #[test]
            fn realloc_run_before_allocated_page_should_move() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let small = allocator.alloc(&mut writer, 16).unwrap();
                let pos = allocator.alloc(&mut writer, 1024).unwrap();
                let last = allocator.alloc(&mut writer, 1024).unwrap();

                let new_pos = allocator.realloc(&mut writer, pos, 1024, 2048).unwrap();

                assert_eq!(new_pos, 8 * 1024);
                verify(
                    &allocator,
                    &writer,
                    &[(small, 16), (last, 1024), (new_pos, 2048)],
                );
            }

            #[test]
            fn realloc_across_classes_should_move_bytes() {
                let mut storage = heap();
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                let mut allocator = Allocator::new(DEFAULT_PAGE_SHIFT);
                let pos = allocator.alloc(&mut writer, 10).unwrap();
                writer.write(pos, &pattern(1, 10)).unwrap();

                let large = allocator.realloc(&mut writer, pos, 10, 600).unwrap();
                let small = allocator.realloc(&mut writer, large, 600, 20).unwrap();

                assert_eq!(large % 1024, 0);
                assert_eq!(bytes(&writer, small, 10), pattern(1, 10));
                verify(&allocator, &writer, &[(small, 20)]);
            }
        }
    }
}
