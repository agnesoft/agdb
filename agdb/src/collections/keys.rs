use crate::DbError;
use crate::DbErrorType;
use crate::DbValue;
use crate::collections::array::ARRAY_SIZE;
use crate::collections::array::SegmentedArray;
use crate::collections::table::Kind;
use crate::collections::table::Lookup;
use crate::collections::table::TABLE_SIZE;
use crate::collections::table::TableHeader;
use crate::collections::table::fmix64;
use crate::collections::table::stored_hash;
use crate::collections::value;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::alloc::u32_at;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;
use crate::utilities::stable_hash::StableHash;

/// `[hash u64][key id u32]`
const KEYS: Kind = Kind {
    size: 12,
    hash: stored_hash,
};
// A key is the record of its id: [key DbValueIndex][refs u64][hash u64].
// A free record has no refs and the next free id in place of the hash.
const RECORD_SHIFT: u32 = 5;
const RECORD: usize = 1 << RECORD_SHIFT;
/// The bytes of encoded [`Keys`].
pub(crate) const KEYS_SIZE: usize = ARRAY_SIZE + 16 + TABLE_SIZE;

/// The id of a key in the [`Keys`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyId(pub(crate) u32);

/// The key of the alias of a node, which has no record.
pub(crate) const ALIAS: KeyId = KeyId(0);
/// The highest key id, the top bit marks labels in the key-values.
pub(crate) const MAX_KEY_ID: u32 = u32::MAX >> 1;

struct Record {
    key: DbValueIndex,
    refs: u64,
    hash: u64,
}

fn slot_id(slot: &[u8]) -> KeyId {
    KeyId(u32_at(slot, 8))
}

fn record_bytes(key: &DbValueIndex, refs: u64, hash: u64) -> [u8; RECORD] {
    let mut bytes = [0_u8; RECORD];
    bytes[..16].copy_from_slice(&key.data());
    bytes[16..24].copy_from_slice(&refs.to_le_bytes());
    bytes[24..].copy_from_slice(&hash.to_le_bytes());
    bytes
}

/// The dictionary of the keys of the values, each stored once under an
/// id and freed with its last reference. The records are in an array by
/// id, the slot 0 being [`ALIAS`], and a table maps the hash of a
/// key to its id.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Keys {
    array: SegmentedArray,
    /// The slots handed out including the slot 0.
    len: u64,
    free: u64,
    table: TableHeader,
}

impl Keys {
    pub(crate) fn new(page_shift: u32) -> Self {
        Self {
            array: SegmentedArray::new(page_shift, RECORD_SHIFT),
            len: 1,
            free: 0,
            table: TableHeader::default(),
        }
    }

    pub(crate) fn decode(bytes: &[u8], page_shift: u32) -> Self {
        let mut keys = Self::new(page_shift);
        keys.array.decode(&bytes[..ARRAY_SIZE]);
        let words = &bytes[ARRAY_SIZE..];
        keys.len = u64_at(words, 0);
        keys.free = u64_at(words, 1);
        keys.table = TableHeader::decode(&words[16..]);
        keys
    }

    pub(crate) fn encode(&self) -> [u8; KEYS_SIZE] {
        let mut bytes = [0_u8; KEYS_SIZE];
        bytes[..ARRAY_SIZE].copy_from_slice(&self.array.encode());
        bytes[ARRAY_SIZE..ARRAY_SIZE + 8].copy_from_slice(&self.len.to_le_bytes());
        bytes[ARRAY_SIZE + 8..ARRAY_SIZE + 16].copy_from_slice(&self.free.to_le_bytes());
        bytes[ARRAY_SIZE + 16..].copy_from_slice(&self.table.encode());
        bytes
    }

    pub(crate) fn len(&self) -> u64 {
        self.table.len()
    }

    /// The id of the `key`, `None` if it is not referenced.
    pub(crate) fn id(&self, view: &impl View, key: &DbValue) -> Result<Option<KeyId>, DbError> {
        let (kind, bytes) = value::encode(key);

        Ok(
            match self.find(view, kind, &bytes, fmix64(key.stable_hash()))? {
                Lookup::Found(_, slot) => Some(slot_id(&slot)),
                Lookup::Missing(_) => None,
            },
        )
    }

    pub(crate) fn key(&self, view: &impl View, id: KeyId) -> Result<DbValue, DbError> {
        value::load(view, &self.record(view, id)?.key)
    }

    /// Adds a reference to the `key` returning its id, storing it under a
    /// new id if it has none.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: &DbValue,
    ) -> Result<KeyId, DbError> {
        if *key == DbValue::None {
            return Err(DbError::db(
                DbErrorType::NotAllowed,
                "Key cannot be none".to_string(),
            ));
        }

        let (kind, bytes) = value::encode(key);
        let hash = fmix64(key.stable_hash());

        let end = match self.find(view, kind, &bytes, hash)? {
            Lookup::Found(_, slot) => {
                let id = slot_id(&slot);
                self.acquire(view, id)?;
                return Ok(id);
            }
            Lookup::Missing(end) => end,
        };

        let id = self.take(view, alloc)?;
        let stored = value::store_bytes(view, alloc, kind, &bytes)?;
        view.write(self.pos(id), &record_bytes(&stored, 1, hash))?;
        let mut slot = [0_u8; KEYS.size as usize];
        slot[..8].copy_from_slice(&hash.to_le_bytes());
        slot[8..].copy_from_slice(&id.0.to_le_bytes());
        self.table.insert_at(view, alloc, &KEYS, end, &slot)?;
        Ok(id)
    }

    /// Adds a reference to the key `id`.
    pub(crate) fn acquire(&self, view: &mut impl ViewMut, id: KeyId) -> Result<(), DbError> {
        let refs = self.record(view, id)?.refs + 1;
        view.write(self.pos(id) + 16, &refs.to_le_bytes())
    }

    /// Removes a reference to the key `id`, freeing the key with its last
    /// one.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: KeyId,
    ) -> Result<(), DbError> {
        let record = self.record(view, id)?;

        if 1 < record.refs {
            return view.write(self.pos(id) + 16, &(record.refs - 1).to_le_bytes());
        }

        let mut probe = self.table.probe(view, &KEYS, record.hash)?;
        let mut found = None;

        while let Some((at, slot)) = probe.next(&KEYS) {
            if slot_id(&slot) == id {
                found = Some(at);
            }
        }

        if let Some(at) = found {
            self.table.remove(view, alloc, &KEYS, at)?;
        }

        value::free(view, alloc, &record.key)?;
        let free = record_bytes(&DbValueIndex::new(), 0, self.free);
        view.write(self.pos(id), &free)?;
        self.free = u64::from(id.0);
        Ok(())
    }

    fn find(&self, view: &impl View, kind: u8, bytes: &[u8], hash: u64) -> Result<Lookup, DbError> {
        let mut probe = self.table.probe(view, &KEYS, hash)?;

        while let Some((at, slot)) = probe.next(&KEYS) {
            let record = self.record(view, slot_id(&slot))?;

            if value::equals_bytes(view, &record.key, kind, bytes)? {
                return Ok(Lookup::Found(at, slot));
            }
        }

        Ok(Lookup::Missing(probe.end()))
    }

    fn pos(&self, id: KeyId) -> u64 {
        self.array.pos(u64::from(id.0))
    }

    /// The record of a referenced key.
    fn record(&self, view: &impl View, id: KeyId) -> Result<Record, DbError> {
        let slot = u64::from(id.0);

        if slot != 0 && slot < self.len {
            let bytes = view.read(self.pos(id), RECORD as u64)?;
            let mut key = [0_u8; 16];
            key.copy_from_slice(&bytes[..16]);
            let record = Record {
                key: DbValueIndex::from_data(key),
                refs: u64_at(&bytes, 2),
                hash: u64_at(&bytes, 3),
            };

            if record.refs != 0 {
                return Ok(record);
            }
        }

        Err(DbError::db(
            DbErrorType::InvalidIndex,
            format!("Invalid key id '{}'", id.0),
        ))
    }

    /// Takes the id of the free list head or a new one at the end.
    fn take(&mut self, view: &mut impl ViewMut, alloc: &mut Allocator) -> Result<KeyId, DbError> {
        if self.free != 0 {
            let id = KeyId(self.free as u32);
            self.free = u64_at(&view.read(self.pos(id), RECORD as u64)?, 3);
            return Ok(id);
        }

        if u64::from(MAX_KEY_ID) < self.len {
            return Err(DbError::db(
                DbErrorType::OutOfBounds,
                format!("Too many keys ({})", MAX_KEY_ID),
            ));
        }

        if self.array.capacity() <= self.len {
            self.array.grow(view, alloc)?;
        }

        self.len += 1;
        Ok(KeyId((self.len - 1) as u32))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::array::tests::allocations as array_allocations;
    use crate::collections::table::tests::allocations as table_allocations;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::allocation;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::values;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeMap;

    /// The ids and references of the keys.
    type Model = BTreeMap<DbValue, (KeyId, u64)>;

    /// The allocations of the `keys`: the array, the table and the stored
    /// keys.
    pub(crate) fn allocations(keys: &Keys, view: &impl View) -> Vec<(u64, u64)> {
        let mut live = array_allocations(&keys.array);
        live.extend(table_allocations(&keys.table, view, &KEYS));

        for slot in keys
            .table
            .slots(view, &KEYS)
            .unwrap()
            .chunks(KEYS.size as usize)
        {
            live.extend(allocation(&keys.record(view, slot_id(slot)).unwrap().key));
        }

        live
    }

    /// The references of the key `id`.
    pub(crate) fn refs(keys: &Keys, view: &impl View, id: KeyId) -> u64 {
        keys.record(view, id).unwrap().refs
    }

    fn assert_keys(view: &TestView, keys: &Keys, model: &Model) {
        assert_eq!(&Keys::decode(&keys.encode(), DEFAULT_PAGE_SHIFT), keys);
        assert_eq!(keys.len(), model.len() as u64);

        for key in values() {
            let expected = model.get(&key).map(|entry| entry.0);
            assert_eq!(keys.id(view, &key).unwrap(), expected, "{key:?}");
        }

        for (key, (id, refs)) in model {
            assert_eq!(&keys.key(view, *id).unwrap(), key);
            assert_eq!(keys.record(view, *id).unwrap().refs, *refs);
        }
    }

    fn random_op(
        view: &mut TestView,
        alloc: &mut Allocator,
        keys: &mut Keys,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let pool = values()
            .into_iter()
            .filter(|key| *key != DbValue::None)
            .collect::<Vec<_>>();
        let key = &pool[random.next(pool.len() as u64) as usize];

        match (random.next(3), model.get_mut(key)) {
            (0, Some((id, refs))) => {
                keys.acquire(view, *id).unwrap();
                *refs += 1;
            }
            (1, Some((id, refs))) => {
                keys.remove(view, alloc, *id).unwrap();
                *refs -= 1;

                if *refs == 0 {
                    model.remove(key);
                }
            }
            (_, entry) => {
                let id = keys.insert(view, alloc, key).unwrap();

                match entry {
                    Some((known, refs)) => {
                        assert_eq!(id, *known);
                        *refs += 1;
                    }
                    None => {
                        assert!(model.values().all(|entry| entry.0 != id));
                        model.insert(key.clone(), (id, 1));
                    }
                }
            }
        }
    }

    mod keys {
        use super::*;

        #[test]
        fn keys_should_hold_keys_over_random_operations() {
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let mut model = Model::new();

                    for _ in 0..500 {
                        random_op(view, alloc, &mut keys, &mut model, &mut random);
                        assert_keys(view, &keys, &model);
                    }

                    verify(alloc, view, &allocations(&keys, view));
                });
            }
        }

        #[test]
        fn keys_failure_should_fail() {
            let long = DbValue::from("a key longer than 15 bytes");
            assert_failures(
                |view, alloc| {
                    let pos = alloc.alloc(view, 1024).unwrap();
                    alloc.alloc(view, 100).unwrap();
                    alloc.free(view, pos, 1024).unwrap();
                    Keys::new(DEFAULT_PAGE_SHIFT)
                },
                |view, alloc, mut keys| {
                    keys.insert(view, alloc, &1.into())?;
                    let id = keys.insert(view, alloc, &long)?;
                    keys.insert(view, alloc, &long)?;
                    keys.acquire(view, id)?;
                    keys.id(view, &long)?;
                    keys.key(view, id)?;
                    keys.remove(view, alloc, KeyId(1))?;
                    keys.remove(view, alloc, id)?;
                    keys.remove(view, alloc, id)?;
                    keys.remove(view, alloc, id)?;
                    keys.insert(view, alloc, &2.into())?;
                    Ok(())
                },
            );
        }

        mod id {
            use super::*;

            #[test]
            fn id_with_colliding_hash_should_compare_key() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let id = keys.insert(view, alloc, &"a".into()).unwrap();
                    let stored = value::store(view, alloc, &"b".into()).unwrap();
                    let hash = keys.record(view, id).unwrap().hash;
                    // WHY: the record of "a" now holds "b", as if the two keys had the same hash
                    view.write(keys.pos(id), &record_bytes(&stored, 1, hash))
                        .unwrap();
                    assert_eq!(keys.id(view, &"a".into()).unwrap(), None);
                    let other = keys.insert(view, alloc, &"a".into()).unwrap();
                    assert_ne!(other, id);
                    keys.remove(view, alloc, id).unwrap();
                    assert_eq!(keys.id(view, &"a".into()).unwrap(), Some(other));
                });
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_reuse_freed_ids() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let ids = (0..3)
                        .map(|key| keys.insert(view, alloc, &key.into()).unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(ids, [KeyId(1), KeyId(2), KeyId(3)]);
                    keys.remove(view, alloc, ids[0]).unwrap();
                    keys.remove(view, alloc, ids[2]).unwrap();
                    assert_eq!(keys.insert(view, alloc, &"x".into()).unwrap(), ids[2]);
                    assert_eq!(keys.insert(view, alloc, &"y".into()).unwrap(), ids[0]);
                    assert_eq!(keys.insert(view, alloc, &"z".into()).unwrap(), KeyId(4));
                });
            }

            #[test]
            fn insert_past_last_id_should_fail() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    keys.len = u64::from(MAX_KEY_ID) + 1;
                    let error = keys.insert(view, alloc, &1.into()).unwrap_err();
                    assert_eq!(error.description, "Too many keys (2147483647)");
                    verify(alloc, view, &[]);
                });
            }

            #[test]
            fn insert_none_should_fail() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let error = keys.insert(view, alloc, &DbValue::None).unwrap_err();
                    assert_eq!(error.description, "Key cannot be none");
                    verify(alloc, view, &[]);
                });
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_key_missing_from_table_should_free_record() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let id = keys.insert(view, alloc, &"a".into()).unwrap();
                    let stored = keys.record(view, id).unwrap().key;
                    // WHY: a record whose hash has no slot of its id stands for a corrupt table
                    view.write(keys.pos(id), &record_bytes(&stored, 1, 0))
                        .unwrap();
                    keys.remove(view, alloc, id).unwrap();
                    assert_eq!(keys.len(), 1);
                    assert!(keys.key(view, id).is_err());
                });
            }
        }

        mod key {
            use super::*;

            #[test]
            fn key_of_invalid_id_should_fail() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let id = keys.insert(view, alloc, &1.into()).unwrap();
                    keys.insert(view, alloc, &2.into()).unwrap();
                    keys.remove(view, alloc, id).unwrap();

                    for id in [0, id.0, 3] {
                        let error = keys.key(view, KeyId(id)).unwrap_err();
                        assert_eq!(error.description, format!("Invalid key id '{id}'"));
                    }
                });
            }
        }
    }
}
