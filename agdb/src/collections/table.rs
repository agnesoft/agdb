use crate::DbError;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;

/// The largest slot of a [`Kind`].
pub(crate) const MAX_SLOT: usize = 48;
/// The bytes of an encoded [`TableHeader`].
pub(crate) const TABLE_SIZE: usize = 24;
const MIN_CAP: u64 = 4;
const ZEROES: [u8; MAX_SLOT] = [0; MAX_SLOT];

pub(crate) type Slot = [u8; MAX_SLOT];

/// The slots of a table: their size and the hash of the key of a slot.
/// An empty slot is all zeroes, which a slot in use never is.
pub(crate) struct Kind {
    pub(crate) size: u64,
    pub(crate) hash: fn(&[u8]) -> u64,
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

fn is_empty(slot: &[u8]) -> bool {
    slot.iter().all(|byte| *byte == 0)
}

/// An open addressing hash table of `cap` (a power of two) slots in one
/// allocation, probed linearly from `hash & (cap - 1)`. It holds at
/// most 15/16 of its slots and halves below 7/16. No entry is further
/// than `dist` from its slot, and the deletion shifts the following
/// entries back, so a probe stops at the first empty slot or past
/// `dist`.
///
/// Encoded as `[pos u64][len u64][cap u32][dist u32]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TableHeader {
    pos: u64,
    len: u64,
    cap: u64,
    dist: u64,
}

/// The slots an entry of `hash` may be in.
pub(crate) struct Probe {
    at: u64,
    left: u64,
    hash: u64,
}

impl TableHeader {
    pub(crate) fn decode(bytes: &[u8]) -> Self {
        let words = u64_at(bytes, 2);

        Self {
            pos: u64_at(bytes, 0),
            len: u64_at(bytes, 1),
            cap: words & u64::from(u32::MAX),
            dist: words >> 32,
        }
    }

    pub(crate) fn encode(&self) -> [u8; TABLE_SIZE] {
        let mut bytes = [0_u8; TABLE_SIZE];
        bytes[0..8].copy_from_slice(&self.pos.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.len.to_le_bytes());
        bytes[16..24].copy_from_slice(&(self.cap | (self.dist << 32)).to_le_bytes());
        bytes
    }

    pub(crate) fn len(&self) -> u64 {
        self.len
    }

    pub(crate) fn probe(&self, hash: u64) -> Probe {
        Probe {
            at: hash & self.cap.wrapping_sub(1),
            left: if self.cap == 0 { 0 } else { self.dist + 1 },
            hash,
        }
    }

    /// The next slot of the probed hash with its position, `None` past
    /// the last one it may be in.
    pub(crate) fn next(
        &self,
        view: &impl View,
        kind: &Kind,
        probe: &mut Probe,
    ) -> Result<Option<(u64, Slot)>, DbError> {
        while probe.left != 0 {
            let at = probe.at;
            let slot = self.slot(view, kind, at)?;

            if is_empty(&slot) {
                break;
            }

            probe.at = (at + 1) & (self.cap - 1);
            probe.left -= 1;

            if (kind.hash)(&slot[..kind.size as usize]) == probe.hash {
                return Ok(Some((at, slot)));
            }
        }

        probe.left = 0;
        Ok(None)
    }

    /// Inserts the `slot`, whose key must not be in the table.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        slot: &[u8],
    ) -> Result<(), DbError> {
        if self.cap * 15 < (self.len + 1) * 16 {
            self.resize(view, alloc, kind, (self.cap * 2).max(MIN_CAP))?;
        }

        let mask = self.cap - 1;
        let mut at = (kind.hash)(slot) & mask;
        let mut dist = 0;

        while !is_empty(&self.slot(view, kind, at)?) {
            at = (at + 1) & mask;
            dist += 1;
        }

        view.write(self.slot_pos(kind, at), slot)?;
        self.len += 1;
        self.dist = self.dist.max(dist);
        Ok(())
    }

    /// Overwrites the slot `at` with one of the same key.
    pub(crate) fn replace(
        &self,
        view: &mut impl ViewMut,
        kind: &Kind,
        at: u64,
        slot: &[u8],
    ) -> Result<(), DbError> {
        view.write(self.slot_pos(kind, at), slot)
    }

    /// Removes the slot `at`, shifting the entries after it back.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        at: u64,
    ) -> Result<(), DbError> {
        if self.len == 1 {
            return self.free(view, alloc, kind);
        }

        let size = kind.size as usize;
        let mask = self.cap - 1;
        let mut hole = at;
        let mut next = (at + 1) & mask;

        loop {
            let slot = self.slot(view, kind, next)?;

            if is_empty(&slot) {
                break;
            }

            let home = (kind.hash)(&slot[..size]) & mask;

            if next.wrapping_sub(hole) & mask <= next.wrapping_sub(home) & mask {
                view.write(self.slot_pos(kind, hole), &slot[..size])?;
                hole = next;
            }

            next = (next + 1) & mask;
        }

        view.write(self.slot_pos(kind, hole), &ZEROES[..size])?;
        self.len -= 1;

        if MIN_CAP < self.cap && self.len * 16 < self.cap * 7 {
            self.resize(view, alloc, kind, self.cap / 2)?;
        }

        Ok(())
    }

    /// The slots in use in slot order, one after another.
    pub(crate) fn slots(&self, view: &impl View, kind: &Kind) -> Result<Vec<u8>, DbError> {
        Ok(view
            .read(self.pos, self.cap * kind.size)?
            .chunks(kind.size as usize)
            .filter(|slot| !is_empty(slot))
            .flatten()
            .copied()
            .collect())
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

        *self = Self::default();
        Ok(())
    }

    /// Rehashes the slots in memory into a new allocation of `cap` slots.
    fn resize(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        kind: &Kind,
        cap: u64,
    ) -> Result<(), DbError> {
        let size = kind.size as usize;
        let slots = self.slots(view, kind)?;
        let mut bytes = vec![0_u8; cap as usize * size];
        let mut table = Self {
            pos: 0,
            len: self.len,
            cap,
            dist: 0,
        };

        for slot in slots.chunks(size) {
            let mut at = (kind.hash)(slot) as usize & (cap as usize - 1);
            let mut dist = 0;

            while !is_empty(&bytes[at * size..(at + 1) * size]) {
                at = (at + 1) & (cap as usize - 1);
                dist += 1;
            }

            bytes[at * size..(at + 1) * size].copy_from_slice(slot);
            table.dist = table.dist.max(dist);
        }

        self.free(view, alloc, kind)?;
        table.pos = alloc.alloc(view, cap * kind.size)?;
        view.write(table.pos, &bytes)?;
        *self = table;
        Ok(())
    }

    fn slot(&self, view: &impl View, kind: &Kind, at: u64) -> Result<Slot, DbError> {
        let mut slot = ZEROES;
        slot[..kind.size as usize].copy_from_slice(&view.read(self.slot_pos(kind, at), kind.size)?);
        Ok(slot)
    }

    fn slot_pos(&self, kind: &Kind, at: u64) -> u64 {
        self.pos + at * kind.size
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeSet;

    /// The (pos, size) of the slots of the table.
    pub(crate) fn allocation(table: &TableHeader, kind: &Kind) -> Option<(u64, u64)> {
        (table.cap != 0).then(|| (table.pos, table.cap * kind.size))
    }

    fn id_hash(slot: &[u8]) -> u64 {
        fmix64(u64_at(slot, 0))
    }

    /// Hashes most keys to a few slots for long clusters that wrap.
    fn clustered_hash(slot: &[u8]) -> u64 {
        u64_at(slot, 0) % 3 + 6
    }

    const IDS: Kind = Kind {
        size: 8,
        hash: id_hash,
    };
    const CLUSTERED: Kind = Kind {
        size: 8,
        hash: clustered_hash,
    };

    fn find_slot(
        table: &TableHeader,
        view: &TestView,
        kind: &Kind,
        key: u64,
    ) -> Result<Option<u64>, DbError> {
        let mut probe = table.probe((kind.hash)(&key.to_le_bytes()));

        while let Some((at, slot)) = table.next(view, kind, &mut probe)? {
            if u64_at(&slot, 0) == key {
                return Ok(Some(at));
            }
        }

        Ok(None)
    }

    fn find(table: &TableHeader, view: &TestView, kind: &Kind, key: u64) -> Option<u64> {
        find_slot(table, view, kind, key).unwrap()
    }

    /// Asserts that the table holds exactly the `keys`, each reachable
    /// from its slot within `dist` and without an empty slot between.
    fn assert_table(table: &TableHeader, view: &TestView, kind: &Kind, keys: &BTreeSet<u64>) {
        let slots = table.slots(view, kind).unwrap();
        let stored = slots
            .chunks(8)
            .map(|slot| u64_at(slot, 0))
            .collect::<BTreeSet<_>>();
        assert_eq!(&stored, keys);
        assert_eq!(table.len, keys.len() as u64);
        assert!(table.cap == 0 || table.len * 16 <= table.cap * 15);

        for key in keys {
            let at = find(table, view, kind, *key).unwrap();
            let home = (kind.hash)(&key.to_le_bytes()) & (table.cap - 1);
            assert!(at.wrapping_sub(home) & (table.cap - 1) <= table.dist);
        }
    }

    fn random_ops(kind: &Kind, seed: u64) {
        with_view(|view, alloc| {
            let mut random = XorShift(seed);
            let mut table = TableHeader::default();
            let mut keys = BTreeSet::new();

            for _ in 0..2000 {
                let key = 1 + random.next(300);

                if let Some(at) = find(&table, view, kind, key) {
                    table.remove(view, alloc, kind, at).unwrap();
                    keys.remove(&key);
                } else {
                    table.insert(view, alloc, kind, &key.to_le_bytes()).unwrap();
                    keys.insert(key);
                }

                assert_table(&table, view, kind, &keys);
                assert_eq!(find(&table, view, kind, 1000), None);
            }

            verify(
                alloc,
                view,
                &allocation(&table, kind).into_iter().collect::<Vec<_>>(),
            );
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
                random_ops(&CLUSTERED, seed);
            }
        }

        #[test]
        fn table_header_failure_should_fail() {
            assert_failures(
                |view, alloc| {
                    let mut table = TableHeader::default();

                    for key in 1..=15 {
                        table
                            .insert(view, alloc, &CLUSTERED, &u64::to_le_bytes(key))
                            .unwrap();
                    }

                    table
                },
                |view, alloc, mut table| {
                    table.insert(view, alloc, &CLUSTERED, &u64::to_le_bytes(16))?;

                    for key in 1..=16 {
                        let at = find_slot(&table, view, &CLUSTERED, key)?.unwrap();
                        table.remove(view, alloc, &CLUSTERED, at)?;
                    }

                    Ok(())
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
                    cap: 4,
                    dist: 2,
                };
                assert_eq!(TableHeader::decode(&table.encode()), table);
            }
        }

        mod next {
            use super::*;

            #[test]
            fn next_in_empty_table_should_find_nothing() {
                with_view(|view, _| {
                    let table = TableHeader::default();
                    let mut probe = table.probe(5);
                    assert_eq!(table.next(view, &IDS, &mut probe).unwrap(), None);
                    assert_eq!(table.next(view, &IDS, &mut probe).unwrap(), None);
                });
            }

            #[test]
            fn next_should_stop_past_dist() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();

                    for key in [6, 9, 7] {
                        table
                            .insert(view, alloc, &CLUSTERED, &u64::to_le_bytes(key))
                            .unwrap();
                    }

                    // WHY: 6 and 9 take slots 2 and 3 of 4 and 7 wraps to slot 0, so a probe of slot 3 ends past `dist` before the empty slot 1
                    let mut probe = table.probe(3);
                    assert_eq!(table.next(view, &CLUSTERED, &mut probe).unwrap(), None);
                    assert_eq!(table.dist, 1);
                });
            }
        }

        mod replace {
            use super::*;

            #[test]
            fn replace_should_overwrite_slot() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();
                    table
                        .insert(view, alloc, &CLUSTERED, &u64::to_le_bytes(1))
                        .unwrap();
                    let at = find(&table, view, &CLUSTERED, 1).unwrap();
                    table
                        .replace(view, &CLUSTERED, at, &u64::to_le_bytes(4))
                        .unwrap();
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
                    table
                        .insert(view, alloc, &IDS, &u64::to_le_bytes(1))
                        .unwrap();
                    let at = find(&table, view, &IDS, 1).unwrap();
                    table.remove(view, alloc, &IDS, at).unwrap();
                    assert_eq!(table, TableHeader::default());
                    verify(alloc, view, &[]);
                });
            }

            #[test]
            fn remove_should_halve_table_below_7_16() {
                with_view(|view, alloc| {
                    let mut table = TableHeader::default();

                    for key in 1..=8 {
                        table
                            .insert(view, alloc, &IDS, &u64::to_le_bytes(key))
                            .unwrap();
                    }

                    assert_eq!(table.cap, 16);

                    for key in 1..=2 {
                        let at = find(&table, view, &IDS, key).unwrap();
                        table.remove(view, alloc, &IDS, at).unwrap();
                    }

                    assert_eq!(table.cap, 8);
                    assert_table(&table, view, &IDS, &(3..=8).collect());
                });
            }
        }
    }
}
