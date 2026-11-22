use crate::DbError;
use crate::storage::StorageSlice;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;

/// The largest slot of a [`Kind`].
pub(crate) const MAX_SLOT: usize = 48;
/// The bytes of an encoded [`TableHeader`].
pub(crate) const TABLE_SIZE: usize = 24;
/// The bytes of a bucket: the smallest page, so that a bucket never
/// crosses a page.
const BUCKET: u64 = 1024;
/// `[next u64][count u64]` of a bucket.
const HEADER: u64 = 16;
/// The directory entries, more segments than any file can hold.
const SEGMENTS: u64 = 40;
const MIN_CAP: u64 = 4;
/// A bucket splits when the average bucket would be fuller than
/// `LOAD.0 / LOAD.1`, and merges below half of that.
const LOAD: (u64, u64) = (3, 4);
const ZEROES: [u8; MAX_SLOT] = [0; MAX_SLOT];

pub(crate) type Slot = [u8; MAX_SLOT];

/// The slots of a table: their size and the hash of the key of a slot.
pub(crate) struct Kind {
    pub(crate) size: u64,
    pub(crate) hash: fn(&[u8]) -> u64,
}

impl Kind {
    /// The slots of a bucket, each with a byte of its hash.
    fn bucket_cap(&self) -> u64 {
        (BUCKET - HEADER) / (self.size + 1)
    }

    /// The position of the first slot in a bucket.
    fn slots_at(&self) -> u64 {
        HEADER + self.bucket_cap()
    }
}

/// The byte of a hash that a bucket keeps for each slot, above the bits
/// that address the buckets.
fn print(hash: u64) -> u8 {
    (hash >> 56) as u8
}

/// The murmur3 finalizer: spreads the stable hashes, which are the
/// values themselves for integers, over the slots.
pub(crate) fn fmix64(mut hash: u64) -> u64 {
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ (hash >> 33)
}

/// The hash kept in the first 8 bytes of a slot.
pub(crate) fn stored_hash(slot: &[u8]) -> u64 {
    u64_at(slot, 0)
}

/// A hash table that grows and shrinks by a bucket at a time. Up to a
/// bucket of slots it is one allocation of `cap` slots. Then `pos` is a
/// directory of segments of buckets, the `k`-th of `2^k` buckets, which
/// never move, and the slots are in the bucket of the low bits of their
/// hash (linear hashing). A bucket is
/// `[next u64][count u64][print u8 × cap][slots]`, `next` being a bucket
/// of its overflow and a print the top byte of the hash of a slot, so
/// that a probe reads only the slots of matching prints.
///
/// Encoded as `[pos u64][len u64][buckets u48 | cap u16]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TableHeader {
    pos: u64,
    len: u64,
    buckets: u64,
    cap: u64,
}

/// The slots of a hash: those of its bucket and its overflow buckets.
pub(crate) struct Probe<'a> {
    hash: u64,
    print: u8,
    first: Page<'a>,
    overflow: Vec<Page<'a>>,
    page: usize,
    index: u64,
}

/// The `count` slots at `base` of the `bytes` read at `pos`, with their
/// prints at `prints` unless it is 0.
struct Page<'a> {
    pos: u64,
    prints: u64,
    base: u64,
    count: u64,
    bytes: StorageSlice<'a>,
}

/// Where the probe of a hash ended, to insert a slot of the hash there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct End {
    hash: u64,
    page: u64,
    count: u64,
}

/// A slot found with its position, or the end of the probe.
pub(crate) enum Lookup {
    Found(u64, Slot),
    Missing(End),
}

impl Page<'_> {
    fn bucket<'a>(view: &'a impl View, kind: &Kind, pos: u64) -> Result<Page<'a>, DbError> {
        let bytes = view.read(pos, BUCKET)?;

        Ok(Page {
            pos,
            prints: HEADER,
            base: kind.slots_at(),
            count: u64_at(&bytes, 1),
            bytes,
        })
    }

    fn next(&self) -> u64 {
        if self.prints != 0 {
            u64_at(&self.bytes, 0)
        } else {
            0
        }
    }
}

impl Probe<'_> {
    /// The next slot of the probed hash with its position.
    pub(crate) fn next(&mut self, kind: &Kind) -> Option<(u64, Slot)> {
        loop {
            let page = if self.page == 0 {
                &self.first
            } else {
                self.overflow.get(self.page - 1)?
            };

            if self.index == page.count {
                self.page += 1;
                self.index = 0;
                continue;
            }

            let index = self.index;
            self.index += 1;

            if page.prints != 0 && page.bytes[(page.prints + index) as usize] != self.print {
                continue;
            }

            let at = page.base + index * kind.size;
            let bytes = &page.bytes[at as usize..(at + kind.size) as usize];

            if (kind.hash)(bytes) == self.hash {
                let mut slot = ZEROES;
                slot[..bytes.len()].copy_from_slice(bytes);
                return Some((page.pos + at, slot));
            }
        }
    }

    /// Where a slot of the probed hash would be inserted.
    pub(crate) fn end(&self) -> End {
        let page = self.overflow.last().unwrap_or(&self.first);

        End {
            hash: self.hash,
            page: page.pos,
            count: page.count,
        }
    }
}

impl TableHeader {
    pub(crate) fn decode(bytes: &[u8]) -> Self {
        let words = u64_at(bytes, 2);

        Self {
            pos: u64_at(bytes, 0),
            len: u64_at(bytes, 1),
            buckets: words & ((1 << 48) - 1),
            cap: words >> 48,
        }
    }

    pub(crate) fn encode(&self) -> [u8; TABLE_SIZE] {
        let mut bytes = [0_u8; TABLE_SIZE];
        bytes[0..8].copy_from_slice(&self.pos.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.len.to_le_bytes());
        bytes[16..24].copy_from_slice(&(self.buckets | (self.cap << 48)).to_le_bytes());
        bytes
    }

    pub(crate) fn len(&self) -> u64 {
        self.len
    }

    /// The probe of the slots of the `hash`, reading its buckets.
    pub(crate) fn probe<'a>(
        &self,
        view: &'a impl View,
        kind: &Kind,
        hash: u64,
    ) -> Result<Probe<'a>, DbError> {
        let first = if self.cap != 0 {
            Page {
                pos: self.pos,
                prints: 0,
                base: 0,
                count: self.len,
                bytes: view.read(self.pos, self.len * kind.size)?,
            }
        } else if self.buckets != 0 {
            Page::bucket(view, kind, self.bucket_pos(view, self.bucket(hash))?)?
        } else {
            Page {
                pos: 0,
                prints: 0,
                base: 0,
                count: 0,
                bytes: StorageSlice::Borrowed(&[]),
            }
        };
        let mut overflow = Vec::new();
        let mut next = first.next();

        while next != 0 {
            let page = Page::bucket(view, kind, next)?;
            next = page.next();
            overflow.push(page);
        }

        Ok(Probe {
            hash,
            print: print(hash),
            first,
            overflow,
            page: 0,
            index: 0,
        })
    }

    /// Inserts the `slot`, whose key must not be in the table.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        slot: &[u8],
    ) -> Result<(), DbError> {
        let end = self.probe(view, kind, (kind.hash)(slot))?.end();
        self.insert_at(view, alloc, kind, end, slot)
    }

    /// Inserts the `slot` at the `end` of the probe of its hash, which
    /// found no slot of its key.
    pub(crate) fn insert_at(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        end: End,
        slot: &[u8],
    ) -> Result<(), DbError> {
        if self.buckets == 0 {
            if self.len == self.cap {
                if self.cap == kind.bucket_cap() {
                    self.split_allocation(view, alloc, kind)?;
                    return self.insert(view, alloc, kind, slot);
                }

                self.resize(
                    view,
                    alloc,
                    kind,
                    (self.cap * 2).clamp(MIN_CAP, kind.bucket_cap()),
                )?;
            }

            view.write(self.pos + self.len * kind.size, slot)?;
            self.len += 1;
            return Ok(());
        }

        let mut end = end;

        if self.slots_cap(kind) * LOAD.0 < (self.len + 1) * LOAD.1 {
            self.split(view, alloc, kind)?;
            end = self.probe(view, kind, end.hash)?.end();
        }

        if end.count < kind.bucket_cap() {
            view.write(end.page + HEADER + end.count, &[print(end.hash)])?;
            view.write(end.page + kind.slots_at() + end.count * kind.size, slot)?;
            view.write(end.page + 8, &(end.count + 1).to_le_bytes())?;
        } else {
            let page = alloc.alloc(view, BUCKET)?;
            let mut bytes = vec![0_u8; (kind.slots_at() + kind.size) as usize];
            bytes[8] = 1;
            bytes[HEADER as usize] = print(end.hash);
            bytes[kind.slots_at() as usize..].copy_from_slice(slot);
            view.write(page, &bytes)?;
            view.write(end.page, &page.to_le_bytes())?;
        }

        self.len += 1;
        Ok(())
    }

    /// Overwrites the slot `at` with one of the same key.
    pub(crate) fn replace(
        &self,
        view: &mut impl ViewMut,
        at: u64,
        slot: &[u8],
    ) -> Result<(), DbError> {
        view.write(at, slot)
    }

    /// Removes the slot `at`, moving the last slot of its bucket there.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        at: u64,
    ) -> Result<(), DbError> {
        self.len -= 1;

        if self.buckets == 0 {
            move_bytes(view, self.pos + self.len * kind.size, at, kind.size)?;

            if self.len == 0 {
                return self.free(view, alloc, kind);
            }

            if MIN_CAP < self.cap && self.len * 16 < self.cap * 7 {
                self.resize(view, alloc, kind, (self.cap / 2).max(MIN_CAP))?;
            }

            return Ok(());
        }

        let hash = (kind.hash)(&view.read(at, kind.size)?);
        let mut previous = 0;
        let mut page = self.bucket_pos(view, self.bucket(hash))?;
        let mut hole = page;
        let (mut next, mut count) = bucket_header(view, page)?;

        while next != 0 {
            previous = page;
            page = next;

            if page < at && at < page + BUCKET {
                hole = page;
            }

            (next, count) = bucket_header(view, page)?;
        }

        let index = (at - hole - kind.slots_at()) / kind.size;
        move_bytes(view, page + HEADER + count - 1, hole + HEADER + index, 1)?;
        move_bytes(
            view,
            page + kind.slots_at() + (count - 1) * kind.size,
            at,
            kind.size,
        )?;

        if count == 1 && previous != 0 {
            alloc.free(view, page, BUCKET)?;
            view.write(previous, &0_u64.to_le_bytes())?;
        } else {
            view.write(page + 8, &(count - 1).to_le_bytes())?;
        }

        if self.len * 2 * LOAD.1 < self.slots_cap(kind) * LOAD.0 {
            self.merge(view, alloc, kind)?;
        }

        Ok(())
    }

    /// The slots in use in bucket order, one after another.
    pub(crate) fn slots(&self, view: &impl View, kind: &Kind) -> Result<Vec<u8>, DbError> {
        if self.buckets == 0 {
            return Ok(view.read(self.pos, self.len * kind.size)?.to_vec());
        }

        let mut slots = Vec::new();

        for bucket in 0..self.buckets {
            slots.extend(self.chain(view, kind, bucket)?.0);
        }

        Ok(slots)
    }

    pub(crate) fn free(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
    ) -> Result<(), DbError> {
        if self.cap != 0 {
            alloc.free(view, self.pos, self.cap * kind.size)?;
        }

        if self.buckets != 0 {
            for bucket in 0..self.buckets {
                self.take(view, alloc, kind, bucket)?;
            }

            self.free_buckets(view, alloc)?;
        }

        *self = Self::default();
        Ok(())
    }

    /// Moves the slots of a full allocation into two buckets.
    fn split_allocation(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
    ) -> Result<(), DbError> {
        let slots = self.slots(view, kind)?;
        alloc.free(view, self.pos, self.cap * kind.size)?;
        self.pos = alloc.alloc(view, SEGMENTS * 8)?;
        let segment = alloc.alloc(view, BUCKET)?;
        view.write(self.pos, &segment.to_le_bytes())?;
        self.buckets = 1;
        self.cap = 0;
        self.write_bucket(view, alloc, kind, 0, &slots)?;
        self.split(view, alloc, kind)
    }

    /// Moves the slots of the two last buckets into one allocation.
    fn merge_buckets(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
    ) -> Result<(), DbError> {
        let mut slots = self.take(view, alloc, kind, 0)?;
        slots.extend(self.take(view, alloc, kind, 1)?);
        self.free_buckets(view, alloc)?;
        self.buckets = 0;
        self.cap = kind.bucket_cap();
        self.pos = alloc.alloc(view, self.cap * kind.size)?;
        view.write(self.pos, &slots)
    }

    /// Adds the next bucket with the slots of the bucket it splits.
    fn split(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
    ) -> Result<(), DbError> {
        let new = self.buckets;

        if (new + 1).is_power_of_two() {
            let segment = (new + 1).ilog2();
            let pos = alloc.alloc(view, BUCKET << segment)?;
            view.write(self.pos + u64::from(segment) * 8, &pos.to_le_bytes())?;
        }

        let old = new - (1 << new.ilog2());
        let slots = self.take(view, alloc, kind, old)?;
        self.buckets += 1;
        let (mut kept, mut moved) = (Vec::new(), Vec::new());

        for slot in slots.chunks(kind.size as usize) {
            if self.bucket((kind.hash)(slot)) == old {
                kept.extend_from_slice(slot);
            } else {
                moved.extend_from_slice(slot);
            }
        }

        self.write_bucket(view, alloc, kind, old, &kept)?;
        self.write_bucket(view, alloc, kind, new, &moved)
    }

    /// Removes the last bucket moving its slots to the bucket it split.
    fn merge(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
    ) -> Result<(), DbError> {
        if self.buckets == 2 {
            return self.merge_buckets(view, alloc, kind);
        }

        self.buckets -= 1;
        let last = self.buckets;
        let into = last - (1 << last.ilog2());
        let mut slots = self.take(view, alloc, kind, into)?;
        slots.extend(self.take(view, alloc, kind, last)?);
        self.write_bucket(view, alloc, kind, into, &slots)?;

        if (last + 1).is_power_of_two() {
            self.free_segment(view, alloc, (last + 1).ilog2())?;
        }

        Ok(())
    }

    /// Frees the segments and the directory.
    fn free_buckets(&self, view: &mut impl ViewMut, alloc: &mut Allocator) -> Result<(), DbError> {
        for segment in 0..=self.buckets.ilog2() {
            self.free_segment(view, alloc, segment)?;
        }

        alloc.free(view, self.pos, SEGMENTS * 8)
    }

    fn free_segment(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        segment: u32,
    ) -> Result<(), DbError> {
        let pos = u64_at(&view.read(self.pos + u64::from(segment) * 8, 8)?, 0);
        alloc.free(view, pos, BUCKET << segment)
    }

    /// The slots of the `bucket` and its overflow buckets.
    fn chain(
        &self,
        view: &impl View,
        kind: &Kind,
        bucket: u64,
    ) -> Result<(Vec<u8>, Vec<u64>), DbError> {
        let mut slots = Vec::new();
        let mut overflow = Vec::new();
        let mut page = self.bucket_pos(view, bucket)?;

        loop {
            let bytes = view.read(page, BUCKET)?;
            let end = kind.slots_at() + u64_at(&bytes, 1) * kind.size;
            slots.extend_from_slice(&bytes[kind.slots_at() as usize..end as usize]);
            page = u64_at(&bytes, 0);

            if page == 0 {
                return Ok((slots, overflow));
            }

            overflow.push(page);
        }
    }

    /// The slots of the `bucket`, freeing its overflow buckets.
    fn take(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        bucket: u64,
    ) -> Result<Vec<u8>, DbError> {
        let (slots, overflow) = self.chain(view, kind, bucket)?;

        for page in overflow {
            alloc.free(view, page, BUCKET)?;
        }

        Ok(slots)
    }

    /// Writes the `slots` to the `bucket` and as many overflow buckets as
    /// they need.
    fn write_bucket(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        bucket: u64,
        slots: &[u8],
    ) -> Result<(), DbError> {
        let chunk = (kind.bucket_cap() * kind.size) as usize;
        let pages = slots.len().div_ceil(chunk).max(1);
        let mut next = 0_u64;

        for i in (0..pages).rev() {
            let page = if i == 0 {
                self.bucket_pos(view, bucket)?
            } else {
                alloc.alloc(view, BUCKET)?
            };
            let slots = &slots[(i * chunk).min(slots.len())..((i + 1) * chunk).min(slots.len())];
            let mut bytes = vec![0_u8; kind.slots_at() as usize];
            bytes[..8].copy_from_slice(&next.to_le_bytes());
            bytes[8..16].copy_from_slice(&(slots.len() as u64 / kind.size).to_le_bytes());

            for (j, slot) in slots.chunks(kind.size as usize).enumerate() {
                bytes[HEADER as usize + j] = print((kind.hash)(slot));
            }

            bytes.extend_from_slice(slots);
            view.write(page, &bytes)?;
            next = page;
        }

        Ok(())
    }

    fn resize(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        cap: u64,
    ) -> Result<(), DbError> {
        self.pos = if self.cap == 0 {
            alloc.alloc(view, cap * kind.size)?
        } else {
            alloc.realloc(view, self.pos, self.cap * kind.size, cap * kind.size)?
        };
        self.cap = cap;
        Ok(())
    }

    /// The slots the buckets hold without overflow.
    fn slots_cap(&self, kind: &Kind) -> u64 {
        self.buckets * kind.bucket_cap()
    }

    /// The bucket of the `hash`: its low bits up to the next power of two
    /// of the buckets, one bit fewer for the buckets not split yet.
    fn bucket(&self, hash: u64) -> u64 {
        let level = self.buckets.ilog2();
        let bucket = hash & ((2 << level) - 1);

        if bucket < self.buckets {
            bucket
        } else {
            bucket - (1 << level)
        }
    }

    fn bucket_pos(&self, view: &impl View, bucket: u64) -> Result<u64, DbError> {
        let segment = (bucket + 1).ilog2();
        let pos = u64_at(&view.read(self.pos + u64::from(segment) * 8, 8)?, 0);
        Ok(pos + (bucket + 1 - (1 << segment)) * BUCKET)
    }
}

fn bucket_header(view: &impl View, page: u64) -> Result<(u64, u64), DbError> {
    let bytes = view.read(page, HEADER)?;
    Ok((u64_at(&bytes, 0), u64_at(&bytes, 1)))
}

/// Moves the `len` bytes at `from` to `to` unless they are the same.
fn move_bytes(view: &mut impl ViewMut, from: u64, to: u64, len: u64) -> Result<(), DbError> {
    if from != to {
        let bytes = view.read(from, len)?.to_vec();
        view.write(to, &bytes)?;
    }

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::Reader;
    use crate::storage::pages::tests::XorShift;
    use crate::storage::storage_simulator::tests::SCALE;
    use crate::storage::storage_simulator::tests::Scale;
    use crate::storage::storage_simulator::tests::Simulation;
    use std::collections::BTreeSet;

    /// The (pos, size) of each allocation of the table.
    pub(crate) fn allocations(
        table: &TableHeader,
        view: &impl View,
        kind: &Kind,
    ) -> Vec<(u64, u64)> {
        if table.cap != 0 {
            return vec![(table.pos, table.cap * kind.size)];
        }

        if table.buckets == 0 {
            return Vec::new();
        }

        let mut live = vec![(table.pos, SEGMENTS * 8)];

        for segment in 0..=table.buckets.ilog2() {
            let pos = u64_at(
                &view.read(table.pos + u64::from(segment) * 8, 8).unwrap(),
                0,
            );
            live.push((pos, BUCKET << segment));
        }

        for bucket in 0..table.buckets {
            let (_, overflow) = table.chain(view, kind, bucket).unwrap();
            live.extend(overflow.into_iter().map(|page| (page, BUCKET)));
        }

        live
    }

    fn id_hash(slot: &[u8]) -> u64 {
        fmix64(u64_at(slot, 0))
    }

    /// Hashes the keys to three buckets for long overflow chains.
    fn clustered_hash(slot: &[u8]) -> u64 {
        u64_at(slot, 0) % 3 + 6
    }

    const IDS: Kind = Kind {
        size: 8,
        hash: id_hash,
    };
    /// 20 slots of a bucket.
    const WIDE: Kind = Kind {
        size: 48,
        hash: id_hash,
    };
    const CLUSTERED: Kind = Kind {
        size: 48,
        hash: clustered_hash,
    };

    fn slot(kind: &Kind, key: u64) -> Vec<u8> {
        let mut slot = vec![0_u8; kind.size as usize];
        slot[..8].copy_from_slice(&key.to_le_bytes());
        slot
    }

    fn find_at(
        table: &TableHeader,
        view: &impl View,
        kind: &Kind,
        key: u64,
    ) -> Result<Option<u64>, DbError> {
        let mut probe = table.probe(view, kind, (kind.hash)(&key.to_le_bytes()))?;

        while let Some((at, slot)) = probe.next(kind) {
            if u64_at(&slot, 0) == key {
                return Ok(Some(at));
            }
        }

        Ok(None)
    }

    fn find(table: &TableHeader, view: &impl View, kind: &Kind, key: u64) -> Option<u64> {
        find_at(table, view, kind, key).unwrap()
    }

    /// Inserts the missing `key` at the end of its probe.
    fn insert(
        table: &mut TableHeader,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        key: u64,
    ) -> Result<(), DbError> {
        let end = table
            .probe(view, kind, (kind.hash)(&key.to_le_bytes()))?
            .end();
        table.insert_at(view, alloc, kind, end, &slot(kind, key))
    }

    fn remove(
        table: &mut TableHeader,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        key: u64,
    ) -> Result<(), DbError> {
        let at = find_at(table, view, kind, key)?.unwrap();
        table.remove(view, alloc, kind, at)
    }

    /// Asserts that the table holds exactly the `keys` within its loads,
    /// each in the bucket of its hash.
    fn assert_table(table: &TableHeader, view: &TestView, kind: &Kind, keys: &BTreeSet<u64>) {
        let stored = table
            .slots(view, kind)
            .unwrap()
            .chunks(kind.size as usize)
            .map(|slot| u64_at(slot, 0))
            .collect::<BTreeSet<_>>();
        assert_eq!(&stored, keys);
        assert_eq!(table.len, keys.len() as u64);

        if table.cap != 0 {
            assert_eq!(table.buckets, 0);
            assert!(table.len <= table.cap && table.cap <= kind.bucket_cap());
            assert!(table.cap == MIN_CAP || table.cap * 7 <= table.len * 16);
        } else if table.buckets != 0 {
            assert!(2 <= table.buckets);
            assert!(table.len * LOAD.1 <= table.slots_cap(kind) * LOAD.0);
            assert!(table.slots_cap(kind) * LOAD.0 <= table.len * 2 * LOAD.1);

            for bucket in 0..table.buckets {
                let (_, overflow) = table.chain(view, kind, bucket).unwrap();
                let first = table.bucket_pos(view, bucket).unwrap();

                for page in std::iter::once(first).chain(overflow) {
                    let bytes = view.read(page, BUCKET).unwrap();
                    let count = u64_at(&bytes, 1);
                    assert!(page == first || count != 0);

                    for i in 0..count {
                        let at = (kind.slots_at() + i * kind.size) as usize;
                        let hash = (kind.hash)(&bytes[at..at + kind.size as usize]);
                        assert_eq!(table.bucket(hash), bucket);
                        assert_eq!(bytes[(HEADER + i) as usize], print(hash));
                    }
                }
            }
        } else {
            assert_eq!(table, &TableHeader::default());
        }

        for key in keys {
            assert!(find(table, view, kind, *key).is_some());
        }
    }

    fn random_ops(kind: &Kind, seed: u64) {
        with_view(|view, alloc| {
            let mut random = XorShift(seed);
            let mut table = TableHeader::default();
            let mut keys = BTreeSet::new();

            for _ in 0..2000 {
                let key = 1 + random.next(300);

                if keys.remove(&key) {
                    remove(&mut table, view, alloc, kind, key).unwrap();
                } else {
                    insert(&mut table, view, alloc, kind, key).unwrap();
                    keys.insert(key);
                }

                assert_table(&table, view, kind, &keys);
                assert_eq!(find(&table, view, kind, 1000), None);
            }

            verify(alloc, view, &allocations(&table, view, kind));
        });
    }

    mod fmix64 {
        use super::*;

        #[test]
        fn fmix64_should_spread_strided_keys() {
            let slots = (0..1024)
                .map(|key| fmix64(key * 8) & 1023)
                .collect::<BTreeSet<_>>();
            // WHY: the keys alone would take only 128 of the 1024 slots, random ones about 647
            assert!(600 < slots.len());
        }
    }

    mod table_header {
        use super::*;

        #[test]
        fn table_header_should_hold_keys_over_random_operations() {
            for seed in 1..=5 {
                random_ops(&IDS, seed);
                random_ops(&WIDE, seed);
                random_ops(&CLUSTERED, seed);
            }
        }

        #[test]
        fn table_header_should_grow_and_shrink_back_to_nothing() {
            for kind in [&IDS, &WIDE, &CLUSTERED] {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();
                    let mut keys = BTreeSet::new();

                    for key in 1..=600 {
                        insert(&mut table, view, alloc, kind, key).unwrap();
                        keys.insert(key);
                        assert_table(&table, view, kind, &keys);
                    }

                    verify(alloc, view, &allocations(&table, view, kind));

                    for key in 1..=600 {
                        remove(&mut table, view, alloc, kind, key).unwrap();
                        keys.remove(&key);
                        assert_table(&table, view, kind, &keys);
                    }

                    verify(alloc, view, &[]);
                });
            }
        }

        #[test]
        fn table_header_should_do_flat_io_from_1k_to_100k_keys() {
            let mut simulation = Simulation::new();
            let mut table = TableHeader::default();
            let mut inserts = Scale::default();
            let mut finds = Scale::default();
            let mut removes = Scale::default();

            for key in 0..SCALE {
                simulation.run(&mut inserts, key, |view, alloc| {
                    insert(&mut table, view, alloc, &IDS, key)
                });
                let found = simulation.run(&mut finds, key + 1, |view, _| {
                    find_at(&table, view, &IDS, key / 2)
                });
                assert!(found.is_some());
            }

            for key in 0..SCALE {
                simulation.run(&mut removes, SCALE - key, |view, alloc| {
                    remove(&mut table, view, alloc, &IDS, key)
                });
            }

            inserts.assert_flat();
            finds.assert_flat();
            removes.assert_flat();
            verify(&simulation.alloc, &Reader::new(&simulation.storage), &[]);
        }

        #[test]
        fn table_header_failure_should_fail() {
            // WHY: the multiples of 3 have one hash, so their bucket overflows
            let keys = |range: std::ops::RangeInclusive<u64>| range.map(|key| key * 3);
            assert_failures(
                |view, alloc| {
                    let pages = (0..20)
                        .map(|_| alloc.alloc(view, BUCKET).unwrap())
                        .collect::<Vec<_>>();
                    alloc.alloc(view, BUCKET).unwrap();

                    for page in pages {
                        alloc.free(view, page, BUCKET).unwrap();
                    }

                    let mut table = TableHeader::default();

                    for key in keys(1..=3) {
                        insert(&mut table, view, alloc, &CLUSTERED, key).unwrap();
                    }

                    table
                },
                |view, alloc, mut table| {
                    for key in keys(4..=50) {
                        insert(&mut table, view, alloc, &CLUSTERED, key)?;
                    }

                    table.slots(view, &CLUSTERED)?;

                    for key in keys(1..=50) {
                        remove(&mut table, view, alloc, &CLUSTERED, key)?;
                    }

                    for key in keys(1..=30) {
                        insert(&mut table, view, alloc, &CLUSTERED, key)?;
                    }

                    table.free(view, alloc, &CLUSTERED)
                },
            );
        }

        mod decode {
            use super::*;

            #[test]
            fn decode_should_read_encoded_header() {
                let table = TableHeader {
                    pos: 4096,
                    len: 3,
                    buckets: (1 << 48) - 1,
                    cap: 126,
                };
                assert_eq!(TableHeader::decode(&table.encode()), table);
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_past_bucket_should_split_it_in_two() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();

                    for key in 1..=20 {
                        insert(&mut table, view, alloc, &WIDE, key).unwrap();
                    }

                    assert_eq!((table.buckets, table.cap), (0, 20));
                    insert(&mut table, view, alloc, &WIDE, 21).unwrap();
                    assert_eq!((table.buckets, table.cap), (2, 0));
                    assert_table(&table, view, &WIDE, &(1..=21).collect());
                });
            }
        }

        mod probe {
            use super::*;

            #[test]
            fn probe_of_empty_table_should_find_nothing() {
                with_view(|view, _| {
                    let table = TableHeader::default();
                    let mut probe = table.probe(view, &IDS, 5).unwrap();
                    assert_eq!(probe.next(&IDS), None);
                    assert_eq!(
                        probe.end(),
                        End {
                            hash: 5,
                            page: 0,
                            count: 0
                        }
                    );
                });
            }
        }

        mod replace {
            use super::*;

            #[test]
            fn replace_should_overwrite_slot() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();
                    insert(&mut table, view, alloc, &CLUSTERED, 1).unwrap();
                    let at = find(&table, view, &CLUSTERED, 1).unwrap();
                    table.replace(view, at, &slot(&CLUSTERED, 4)).unwrap();
                    assert_eq!(find(&table, view, &CLUSTERED, 4), Some(at));
                    assert_eq!(find(&table, view, &CLUSTERED, 1), None);
                });
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_last_should_free_table() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();
                    insert(&mut table, view, alloc, &IDS, 1).unwrap();
                    remove(&mut table, view, alloc, &IDS, 1).unwrap();
                    assert_eq!(table, TableHeader::default());
                    assert_eq!(allocations(&table, view, &IDS), []);
                    verify(alloc, view, &[]);
                });
            }

            #[test]
            fn remove_should_halve_allocation_below_7_16() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();

                    for key in 1..=9 {
                        insert(&mut table, view, alloc, &IDS, key).unwrap();
                    }

                    assert_eq!(table.cap, 16);

                    for key in 1..=3 {
                        remove(&mut table, view, alloc, &IDS, key).unwrap();
                    }

                    assert_eq!(table.cap, 8);
                    assert_table(&table, view, &IDS, &(4..=9).collect());
                });
            }
        }

        mod free {
            use super::*;

            #[test]
            fn free_should_release_buckets_and_overflow() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();

                    for key in 1..=200 {
                        insert(&mut table, view, alloc, &CLUSTERED, key).unwrap();
                    }

                    assert!(8 < allocations(&table, view, &CLUSTERED).len());
                    table.free(view, alloc, &CLUSTERED).unwrap();
                    assert_eq!(table, TableHeader::default());
                    verify(alloc, view, &[]);
                });
            }
        }
    }
}
