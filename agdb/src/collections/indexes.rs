use crate::DbError;
use crate::DbId;
use crate::DbValue;
use crate::collections::table::Kind;
use crate::collections::table::Lookup;
use crate::collections::table::Slot;
use crate::collections::table::TableHeader;
use crate::collections::table::fmix64;
use crate::collections::table::stored_hash;
use crate::collections::value;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;
use crate::utilities::stable_hash::StableHash;

/// `[hash u64][value DbValueIndex][ids]`, the ids being a single id as
/// `[0 u64][id i64][0 u64]` or the [`TableHeader`] of an id set.
const VALUES: Kind = Kind {
    size: 48,
    hash: stored_hash,
};
/// `[id i64]`
const IDS: Kind = Kind {
    size: 8,
    hash: id_hash,
};
/// `[key DbValueIndex][values TableHeader][count u64]`
const ENTRY: u64 = 48;
/// The bytes of encoded [`Indexes`].
pub(crate) const INDEXES_SIZE: usize = 16;

fn id_hash(slot: &[u8]) -> u64 {
    fmix64(u64_at(slot, 0))
}

fn value_index(bytes: &[u8]) -> DbValueIndex {
    let mut index = [0_u8; 16];
    index.copy_from_slice(&bytes[..16]);
    DbValueIndex::from_data(index)
}

enum Ids {
    Single(DbId),
    Set(TableHeader),
}

impl Ids {
    fn decode(slot: &[u8]) -> Self {
        if u64_at(slot, 3) == 0 {
            Self::Single(DbId(u64_at(slot, 4) as i64))
        } else {
            Self::Set(TableHeader::decode(&slot[24..48]))
        }
    }

    fn encode(&self, slot: &mut Slot) {
        match self {
            Self::Single(id) => {
                slot[24..32].copy_from_slice(&[0; 8]);
                slot[32..40].copy_from_slice(&id.0.to_le_bytes());
                slot[40..48].copy_from_slice(&[0; 8]);
            }
            Self::Set(set) => slot[24..48].copy_from_slice(&set.encode()),
        }
    }
}

/// The index of the values of a key: a table of the values, each with the
/// ids of the elements having it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Index {
    pub(crate) key: DbValue,
    stored_key: DbValueIndex,
    values: TableHeader,
    /// The number of ids of all values.
    pub(crate) count: u64,
}

/// The indexes, listed in one allocation of entries. They are loaded
/// with the root and each entry is written as it changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Indexes {
    pos: u64,
    list: Vec<Index>,
}

impl Index {
    fn find(&self, view: &impl View, value: &DbValue, hash: u64) -> Result<Lookup, DbError> {
        let (kind, bytes) = value::encode(value);
        let mut probe = self.values.probe(view, &VALUES, hash)?;

        while let Some((at, slot)) = probe.next(&VALUES) {
            if value::equals_bytes(view, &value_index(&slot[8..]), kind, &bytes)? {
                return Ok(Lookup::Found(at, slot));
            }
        }

        Ok(Lookup::Missing(probe.end()))
    }
}

impl Indexes {
    pub(crate) fn load(view: &impl View, bytes: &[u8]) -> Result<Self, DbError> {
        let pos = u64_at(bytes, 0);
        let entries = view.read(pos, u64_at(bytes, 1) * ENTRY)?;
        let mut list = Vec::new();

        for entry in entries.chunks(ENTRY as usize) {
            let stored_key = value_index(entry);
            list.push(Index {
                key: value::load(view, &stored_key)?,
                stored_key,
                values: TableHeader::decode(&entry[16..40]),
                count: u64_at(entry, 5),
            });
        }

        Ok(Self { pos, list })
    }

    pub(crate) fn encode(&self) -> [u8; INDEXES_SIZE] {
        let mut bytes = [0_u8; INDEXES_SIZE];
        bytes[..8].copy_from_slice(&self.pos.to_le_bytes());
        bytes[8..].copy_from_slice(&(self.list.len() as u64).to_le_bytes());
        bytes
    }

    pub(crate) fn list(&self) -> &[Index] {
        &self.list
    }

    pub(crate) fn index(&self, key: &DbValue) -> Option<&Index> {
        self.list.iter().find(|index| index.key == *key)
    }

    /// Creates an empty index of the `key`, which must not have one.
    pub(crate) fn insert_index(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: &DbValue,
    ) -> Result<(), DbError> {
        let stored_key = value::store(view, alloc, key)?;
        let len = self.list.len() as u64;
        self.pos = if len == 0 {
            alloc.alloc(view, ENTRY)?
        } else {
            alloc.realloc(view, self.pos, len * ENTRY, (len + 1) * ENTRY)?
        };
        self.list.push(Index {
            key: key.clone(),
            stored_key,
            values: TableHeader::default(),
            count: 0,
        });
        self.write_entry(view, len as usize)
    }

    /// Removes the index of the `key` returning its number of ids, 0 if
    /// there is none.
    pub(crate) fn remove_index(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: &DbValue,
    ) -> Result<u64, DbError> {
        let Some(i) = self.list.iter().position(|index| index.key == *key) else {
            return Ok(0);
        };

        let mut index = self.list.remove(i);

        for slot in index
            .values
            .slots(view, &VALUES)?
            .chunks(VALUES.size as usize)
        {
            value::free(view, alloc, &value_index(&slot[8..]))?;

            if let Ids::Set(mut set) = Ids::decode(slot) {
                set.free(view, alloc, &IDS)?;
            }
        }

        index.values.free(view, alloc, &VALUES)?;
        value::free(view, alloc, &index.stored_key)?;
        let len = self.list.len() as u64;

        for j in i..self.list.len() {
            self.write_entry(view, j)?;
        }

        if len == 0 {
            alloc.free(view, self.pos, ENTRY)?;
            self.pos = 0;
        } else {
            self.pos = alloc.realloc(view, self.pos, (len + 1) * ENTRY, len * ENTRY)?;
        }

        Ok(index.count)
    }

    /// Adds the `id` to the ids of the `value` in the index of the `key`
    /// if there is one. The `id` must not have the value yet.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: &DbValue,
        value: &DbValue,
        id: DbId,
    ) -> Result<(), DbError> {
        let Some(i) = self.list.iter().position(|index| index.key == *key) else {
            return Ok(());
        };

        let index = &mut self.list[i];
        let hash = fmix64(value.stable_hash());

        match index.find(view, value, hash)? {
            Lookup::Found(at, mut slot) => {
                let mut set = match Ids::decode(&slot) {
                    Ids::Single(other) => {
                        let mut set = TableHeader::default();
                        set.insert(view, alloc, &IDS, &other.0.to_le_bytes())?;
                        set
                    }
                    Ids::Set(set) => set,
                };
                set.insert(view, alloc, &IDS, &id.0.to_le_bytes())?;
                Ids::Set(set).encode(&mut slot);
                index.values.replace(view, at, &slot)?;
            }
            Lookup::Missing(end) => {
                let mut slot = [0_u8; 48];
                slot[..8].copy_from_slice(&hash.to_le_bytes());
                slot[8..24].copy_from_slice(&value::store(view, alloc, value)?.data());
                Ids::Single(id).encode(&mut slot);
                index.values.insert_at(view, alloc, &VALUES, end, &slot)?;
            }
        }

        index.count += 1;
        self.write_entry(view, i)
    }

    /// Removes the `id` from the ids of the `value` in the index of the
    /// `key` if there is one.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: &DbValue,
        value: &DbValue,
        id: DbId,
    ) -> Result<(), DbError> {
        let Some(i) = self.list.iter().position(|index| index.key == *key) else {
            return Ok(());
        };

        let index = &mut self.list[i];

        let Lookup::Found(at, mut slot) = index.find(view, value, fmix64(value.stable_hash()))?
        else {
            return Ok(());
        };

        match Ids::decode(&slot) {
            Ids::Single(other) if other == id => {
                index.values.remove(view, alloc, &VALUES, at)?;
                value::free(view, alloc, &value_index(&slot[8..]))?;
            }
            Ids::Single(_) => return Ok(()),
            Ids::Set(mut set) => {
                let mut probe = set.probe(view, &IDS, fmix64(id.0 as u64))?;

                let Some((id_at, _)) = probe.next(&IDS) else {
                    return Ok(());
                };

                set.remove(view, alloc, &IDS, id_at)?;

                let ids = if set.len() == 1 {
                    let last = set.slots(view, &IDS)?;
                    set.free(view, alloc, &IDS)?;
                    Ids::Single(DbId(u64_at(&last, 0) as i64))
                } else {
                    Ids::Set(set)
                };

                ids.encode(&mut slot);
                index.values.replace(view, at, &slot)?;
            }
        }

        index.count -= 1;
        self.write_entry(view, i)
    }

    /// The ids with the `value` in the index of the `key`, `None` if
    /// there is no such index.
    pub(crate) fn search(
        &self,
        view: &impl View,
        key: &DbValue,
        value: &DbValue,
    ) -> Result<Option<Vec<DbId>>, DbError> {
        let Some(index) = self.index(key) else {
            return Ok(None);
        };

        let Lookup::Found(_, slot) = index.find(view, value, fmix64(value.stable_hash()))? else {
            return Ok(Some(Vec::new()));
        };

        Ok(Some(match Ids::decode(&slot) {
            Ids::Single(id) => vec![id],
            Ids::Set(set) => set
                .slots(view, &IDS)?
                .chunks(8)
                .map(|id| DbId(u64_at(id, 0) as i64))
                .collect(),
        }))
    }

    fn write_entry(&self, view: &mut impl ViewMut, i: usize) -> Result<(), DbError> {
        let index = &self.list[i];
        let mut entry = [0_u8; ENTRY as usize];
        entry[..16].copy_from_slice(&index.stored_key.data());
        entry[16..40].copy_from_slice(&index.values.encode());
        entry[40..].copy_from_slice(&index.count.to_le_bytes());
        view.write(self.pos + i as u64 * ENTRY, &entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::table::tests::allocations as table_allocations;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::allocation;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    type Model = BTreeMap<DbValue, BTreeMap<DbValue, BTreeSet<i64>>>;

    const LONG: &str = "a value longer than 15 bytes";

    fn keys_pool() -> Vec<DbValue> {
        vec!["k".into(), LONG.into(), 1.into()]
    }

    fn values_pool() -> Vec<DbValue> {
        vec![
            1.into(),
            2.into(),
            "v".into(),
            LONG.into(),
            vec![1_u64; 3].into(),
        ]
    }

    fn live(view: &TestView, indexes: &Indexes) -> Vec<(u64, u64)> {
        let mut live = Vec::new();
        let len = indexes.list.len() as u64;
        live.extend((len != 0).then(|| (indexes.pos, len * ENTRY)));

        for index in &indexes.list {
            live.extend(allocation(&index.stored_key));
            live.extend(table_allocations(&index.values, view, &VALUES));

            for slot in index.values.slots(view, &VALUES).unwrap().chunks(48) {
                live.extend(allocation(&value_index(&slot[8..])));

                if let Ids::Set(set) = Ids::decode(slot) {
                    live.extend(table_allocations(&set, view, &IDS));
                }
            }
        }

        live
    }

    fn assert_indexes(view: &TestView, indexes: &Indexes, model: &Model) {
        assert_eq!(&Indexes::load(view, &indexes.encode()).unwrap(), indexes);
        let keys = indexes
            .list()
            .iter()
            .map(|index| index.key.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(keys, model.keys().cloned().collect());

        for key in keys_pool() {
            for value in values_pool() {
                let ids = indexes
                    .search(view, &key, &value)
                    .unwrap()
                    .map(|ids| ids.iter().map(|id| id.0).collect::<BTreeSet<_>>());
                let expected = model
                    .get(&key)
                    .map(|values| values.get(&value).cloned().unwrap_or_default());
                assert_eq!(ids, expected);
            }
        }

        for (key, values) in model {
            let count = values.values().map(|ids| ids.len() as u64).sum::<u64>();
            assert_eq!(indexes.index(key).unwrap().count, count);

            for slot in indexes
                .index(key)
                .unwrap()
                .values
                .slots(view, &VALUES)
                .unwrap()
                .chunks(48)
            {
                let value = value::load(view, &value_index(&slot[8..])).unwrap();
                assert_eq!(
                    matches!(Ids::decode(slot), Ids::Single(_)),
                    values[&value].len() == 1
                );
            }
        }
    }

    fn random_op(
        view: &mut TestView,
        alloc: &mut Allocator,
        indexes: &mut Indexes,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let keys = keys_pool();
        let values = values_pool();
        let key = &keys[random.next(keys.len() as u64) as usize];
        let value = &values[random.next(values.len() as u64) as usize];
        let id = match random.next(20) as i64 - 10 {
            0 => DbId(10),
            id => DbId(id),
        };

        match random.next(20) {
            0 => {
                let count = indexes.remove_index(view, alloc, key).unwrap();
                let expected = model.remove(key).map_or(0, |values| {
                    values.values().map(|ids| ids.len() as u64).sum()
                });
                assert_eq!(count, expected);
            }
            1 if !model.contains_key(key) => {
                indexes.insert_index(view, alloc, key).unwrap();
                model.insert(key.clone(), BTreeMap::new());
            }
            2..=10 => {
                let has = model
                    .get(key)
                    .is_some_and(|values| values.get(value).is_some_and(|ids| ids.contains(&id.0)));

                if !has {
                    indexes.insert(view, alloc, key, value, id).unwrap();

                    if let Some(values) = model.get_mut(key) {
                        values.entry(value.clone()).or_default().insert(id.0);
                    }
                }
            }
            _ => {
                indexes.remove(view, alloc, key, value, id).unwrap();

                if let Some(values) = model.get_mut(key)
                    && let Some(ids) = values.get_mut(value)
                {
                    ids.remove(&id.0);

                    if ids.is_empty() {
                        values.remove(value);
                    }
                }
            }
        }
    }

    mod indexes {
        use super::*;

        #[test]
        fn indexes_should_hold_ids_over_random_operations() {
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let mut indexes = Indexes::default();
                    let mut model = Model::new();

                    for _ in 0..500 {
                        random_op(view, alloc, &mut indexes, &mut model, &mut random);
                        assert_indexes(view, &indexes, &model);
                    }

                    verify(alloc, view, &live(view, &indexes));
                });
            }
        }

        #[test]
        fn indexes_failure_should_fail() {
            let long = DbValue::from(LONG);
            assert_failures(
                |_, _| Indexes::default(),
                |view, alloc, mut indexes| {
                    indexes.insert_index(view, alloc, &long)?;
                    indexes.insert_index(view, alloc, &1.into())?;
                    indexes.insert_index(view, alloc, &2.into())?;

                    for id in 1..=3 {
                        indexes.insert(view, alloc, &long, &long, DbId(id))?;
                    }

                    indexes.search(view, &long, &long)?;
                    Indexes::load(view, &indexes.encode())?;

                    for id in 1..=3 {
                        indexes.remove(view, alloc, &long, &long, DbId(id))?;
                    }

                    indexes.insert(view, alloc, &long, &long, DbId(1))?;
                    indexes.insert(view, alloc, &long, &long, DbId(2))?;
                    indexes.remove_index(view, alloc, &1.into())?;
                    indexes.remove_index(view, alloc, &long)?;
                    indexes.remove_index(view, alloc, &2.into())?;
                    Ok(())
                },
            );
        }

        mod search {
            use super::*;

            #[test]
            fn search_with_colliding_hash_should_compare_value() {
                with_view(|view, alloc| {
                    let mut indexes = Indexes::default();
                    indexes.insert_index(view, alloc, &1.into()).unwrap();
                    indexes
                        .insert(view, alloc, &1.into(), &"a".into(), DbId(1))
                        .unwrap();
                    let hash = fmix64(DbValue::from("b").stable_hash());
                    let mut slot = [0_u8; 48];
                    slot[..8].copy_from_slice(&hash.to_le_bytes());
                    slot[8..24]
                        .copy_from_slice(&value::store(view, alloc, &"c".into()).unwrap().data());
                    Ids::Single(DbId(2)).encode(&mut slot);
                    // WHY: "c" stored under the hash of "b" stands for two values with the same hash
                    indexes.list[0]
                        .values
                        .insert(view, alloc, &VALUES, &slot)
                        .unwrap();
                    assert_eq!(
                        indexes.search(view, &1.into(), &"b".into()).unwrap(),
                        Some(vec![])
                    );
                    indexes
                        .remove(view, alloc, &1.into(), &"b".into(), DbId(2))
                        .unwrap();
                    assert_eq!(
                        indexes.search(view, &1.into(), &"c".into()).unwrap(),
                        Some(vec![])
                    );
                });
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_id_without_value_should_do_nothing() {
                with_view(|view, alloc| {
                    let mut indexes = Indexes::default();
                    indexes.insert_index(view, alloc, &1.into()).unwrap();
                    indexes
                        .insert(view, alloc, &1.into(), &1.into(), DbId(1))
                        .unwrap();
                    indexes
                        .remove(view, alloc, &1.into(), &1.into(), DbId(2))
                        .unwrap();
                    indexes
                        .insert(view, alloc, &1.into(), &2.into(), DbId(1))
                        .unwrap();
                    indexes
                        .insert(view, alloc, &1.into(), &2.into(), DbId(2))
                        .unwrap();
                    indexes
                        .remove(view, alloc, &1.into(), &2.into(), DbId(3))
                        .unwrap();
                    indexes
                        .remove(view, alloc, &2.into(), &2.into(), DbId(1))
                        .unwrap();
                    assert_eq!(indexes.index(&1.into()).unwrap().count, 3);
                    assert_eq!(
                        indexes.search(view, &1.into(), &1.into()).unwrap(),
                        Some(vec![DbId(1)])
                    );
                });
            }
        }
    }
}
