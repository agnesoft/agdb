use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbValue;
use crate::collections::keys::ALIAS;
use crate::collections::keys::KeyId;
use crate::collections::keys::Keys;
use crate::collections::table::Kind;
use crate::collections::table::Lookup;
use crate::collections::table::Slot;
use crate::collections::table::TABLE_SIZE;
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
/// `[key id u64][values TableHeader][count u64]`
const ENTRY: u64 = 40;
/// The bytes of encoded [`Indexes`]: `[pos u64][len u64]` of the list then
/// the values and the count of the aliases.
pub(crate) const INDEXES_SIZE: usize = 16 + TABLE_SIZE + 8;

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

    fn ids(&self, view: &impl View) -> Result<Vec<DbId>, DbError> {
        Ok(match self {
            Self::Single(id) => vec![*id],
            Self::Set(set) => set
                .slots(view, &IDS)?
                .chunks(8)
                .map(|id| DbId(u64_at(id, 0) as i64))
                .collect(),
        })
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
/// ids of the elements having it. It holds a reference to its key.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Index {
    pub(crate) key: DbValue,
    pub(crate) key_id: KeyId,
    values: TableHeader,
    /// The number of ids of all values.
    pub(crate) count: u64,
}

/// The indexes of the keys, listed in one allocation of entries, and the
/// unique index of the aliases, the values of [`ALIAS`]. They are
/// loaded with the root and each entry is written as it changes, the
/// aliases being kept in the root.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Indexes {
    pos: u64,
    list: Vec<Index>,
    aliases: Index,
}

impl Default for Indexes {
    fn default() -> Self {
        Self {
            pos: 0,
            list: Vec::new(),
            aliases: Index::new(DbValue::None, ALIAS),
        }
    }
}

impl Index {
    fn new(key: DbValue, key_id: KeyId) -> Self {
        Self {
            key,
            key_id,
            values: TableHeader::default(),
            count: 0,
        }
    }

    /// The ids with the `value`.
    pub(crate) fn search(&self, view: &impl View, value: &DbValue) -> Result<Vec<DbId>, DbError> {
        let Lookup::Found(_, slot) = self.find(view, value, fmix64(value.stable_hash()))? else {
            return Ok(Vec::new());
        };

        Ids::decode(&slot).ids(view)
    }

    /// The values with their ids in the slot order of the table.
    pub(crate) fn values(&self, view: &impl View) -> Result<Vec<(DbValue, Vec<DbId>)>, DbError> {
        let mut values = Vec::new();

        for slot in self
            .values
            .slots(view, &VALUES)?
            .chunks(VALUES.size as usize)
        {
            let value = value::load(view, &value_index(&slot[8..]))?;
            values.push((value, Ids::decode(slot).ids(view)?));
        }

        Ok(values)
    }

    /// Writes the entry of a listed index at `at`.
    fn write(&self, view: &mut impl ViewMut, at: Option<u64>) -> Result<(), DbError> {
        let Some(at) = at else {
            return Ok(());
        };

        let mut entry = [0_u8; ENTRY as usize];
        entry[..8].copy_from_slice(&u64::from(self.key_id.0).to_le_bytes());
        entry[8..32].copy_from_slice(&self.values.encode());
        entry[32..].copy_from_slice(&self.count.to_le_bytes());
        view.write(at, &entry)
    }

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
    pub(crate) fn load(view: &impl View, keys: &Keys, bytes: &[u8]) -> Result<Self, DbError> {
        let pos = u64_at(bytes, 0);
        let entries = view.read(pos, u64_at(bytes, 1) * ENTRY)?;
        let mut list = Vec::new();

        for entry in entries.chunks(ENTRY as usize) {
            let key_id = KeyId(u64_at(entry, 0) as u32);
            list.push(Index {
                key: keys.key(view, key_id)?,
                key_id,
                values: TableHeader::decode(&entry[8..32]),
                count: u64_at(entry, 4),
            });
        }

        let mut aliases = Index::new(DbValue::None, ALIAS);
        aliases.values = TableHeader::decode(&bytes[16..16 + TABLE_SIZE]);
        aliases.count = u64_at(bytes, 5);
        Ok(Self { pos, list, aliases })
    }

    pub(crate) fn encode(&self) -> [u8; INDEXES_SIZE] {
        let mut bytes = [0_u8; INDEXES_SIZE];
        bytes[..8].copy_from_slice(&self.pos.to_le_bytes());
        bytes[8..16].copy_from_slice(&(self.list.len() as u64).to_le_bytes());
        bytes[16..16 + TABLE_SIZE].copy_from_slice(&self.aliases.values.encode());
        bytes[16 + TABLE_SIZE..].copy_from_slice(&self.aliases.count.to_le_bytes());
        bytes
    }

    pub(crate) fn aliases(&self) -> &Index {
        &self.aliases
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
        keys: &mut Keys,
        key: &DbValue,
    ) -> Result<(), DbError> {
        let key_id = keys.insert(view, alloc, key)?;
        let len = self.list.len() as u64;
        self.pos = if len == 0 {
            alloc.alloc(view, ENTRY)?
        } else {
            alloc.realloc(view, self.pos, len * ENTRY, (len + 1) * ENTRY)?
        };
        self.list.push(Index::new(key.clone(), key_id));
        self.list[len as usize].write(view, Some(self.pos + len * ENTRY))
    }

    /// Removes the index of the `key` returning its number of ids, 0 if
    /// there is none.
    pub(crate) fn remove_index(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
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
        keys.remove(view, alloc, index.key_id)?;
        let len = self.list.len() as u64;

        for j in i..self.list.len() {
            self.list[j].write(view, Some(self.pos + j as u64 * ENTRY))?;
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
    /// if there is one. The `id` must not have the value yet and an alias
    /// must not be taken.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: KeyId,
        value: &DbValue,
        id: DbId,
    ) -> Result<(), DbError> {
        let Some((index, entry)) = self.find_mut(key) else {
            return Ok(());
        };

        let hash = fmix64(value.stable_hash());

        match index.find(view, value, hash)? {
            Lookup::Found(_, slot) if key == ALIAS => {
                return Err(DbError::db(
                    DbErrorType::NotAllowed,
                    format!("Alias '{value}' is taken by '{}'", u64_at(&slot, 4) as i64),
                ));
            }
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
        index.write(view, entry)
    }

    /// Removes the `id` from the ids of the `value` in the index of the
    /// `key` if there is one.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        key: KeyId,
        value: &DbValue,
        id: DbId,
    ) -> Result<(), DbError> {
        let Some((index, entry)) = self.find_mut(key) else {
            return Ok(());
        };

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
        index.write(view, entry)
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

        index.search(view, value).map(Some)
    }

    /// The index of the `key` with the position of its entry, `None` for
    /// the aliases.
    fn find_mut(&mut self, key: KeyId) -> Option<(&mut Index, Option<u64>)> {
        if key == ALIAS {
            return Some((&mut self.aliases, None));
        }

        let i = self.list.iter().position(|index| index.key_id == key)?;
        Some((&mut self.list[i], Some(self.pos + i as u64 * ENTRY)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::keys::tests::allocations as keys_allocations;
    use crate::collections::keys::tests::refs;
    use crate::collections::table::tests::allocations as table_allocations;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::allocation;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    /// The ids by value of each index, the aliases under `DbValue::None`.
    type Model = BTreeMap<DbValue, BTreeMap<DbValue, BTreeSet<i64>>>;

    const LONG: &str = "a value longer than 15 bytes";

    fn keys_pool() -> Vec<DbValue> {
        vec!["k".into(), LONG.into(), 1.into(), DbValue::None]
    }

    fn values_pool() -> Vec<DbValue> {
        vec![
            1.into(),
            2.into(),
            "v".into(),
            LONG.into(),
            vec![1_u64; 3].into(),
            DbValue::None,
        ]
    }

    /// The keys of the pool, each with a reference of the test.
    fn keys(view: &mut TestView, alloc: &mut Allocator) -> Keys {
        let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);

        for key in keys_pool().iter().filter(|key| **key != DbValue::None) {
            keys.insert(view, alloc, key).unwrap();
        }

        keys
    }

    fn key_id(view: &TestView, keys: &Keys, key: &DbValue) -> KeyId {
        if *key == DbValue::None {
            return ALIAS;
        }

        keys.id(view, key).unwrap().unwrap()
    }

    fn model() -> Model {
        Model::from([(DbValue::None, BTreeMap::new())])
    }

    fn index<'a>(indexes: &'a Indexes, key: &DbValue) -> Option<&'a Index> {
        if *key == DbValue::None {
            return Some(indexes.aliases());
        }

        indexes.index(key)
    }

    fn live(view: &TestView, keys: &Keys, indexes: &Indexes) -> Vec<(u64, u64)> {
        let mut live = keys_allocations(keys, view);
        let len = indexes.list.len() as u64;
        live.extend((len != 0).then(|| (indexes.pos, len * ENTRY)));

        for index in indexes.list.iter().chain([&indexes.aliases]) {
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

    fn assert_indexes(view: &TestView, keys: &Keys, indexes: &Indexes, model: &Model) {
        assert_eq!(
            &Indexes::load(view, keys, &indexes.encode()).unwrap(),
            indexes
        );
        let indexed = indexes
            .list()
            .iter()
            .map(|index| index.key.clone())
            .chain([DbValue::None])
            .collect::<BTreeSet<_>>();
        assert_eq!(indexed, model.keys().cloned().collect());
        assert_eq!(
            indexes.search(view, &DbValue::None, &1.into()).unwrap(),
            None
        );

        for key in keys_pool() {
            if key != DbValue::None {
                let indexed = u64::from(model.contains_key(&key));
                assert_eq!(refs(keys, view, key_id(view, keys, &key)), 1 + indexed);
            }

            for value in values_pool() {
                let ids = index(indexes, &key)
                    .map(|index| index.search(view, &value).unwrap())
                    .map(|ids| ids.iter().map(|id| id.0).collect::<BTreeSet<_>>());
                let expected = model
                    .get(&key)
                    .map(|values| values.get(&value).cloned().unwrap_or_default());
                assert_eq!(ids, expected);
            }
        }

        for (key, values) in model {
            let index = index(indexes, key).unwrap();
            let count = values.values().map(|ids| ids.len() as u64).sum::<u64>();
            assert_eq!(index.count, count);
            let mut listed = BTreeMap::new();

            for entry in index.values(view).unwrap() {
                let ids = entry.1.iter().map(|id| id.0).collect::<BTreeSet<_>>();
                assert_eq!(ids.len(), entry.1.len());
                listed.insert(entry.0, ids);
            }

            assert_eq!(&listed, values);

            for slot in index.values.slots(view, &VALUES).unwrap().chunks(48) {
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
        keys: &mut Keys,
        indexes: &mut Indexes,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let pool = keys_pool();
        let values = values_pool();
        let key = &pool[random.next(pool.len() as u64) as usize];
        let value = &values[random.next(values.len() as u64) as usize];
        let key_id = key_id(view, keys, key);
        let id = match random.next(20) as i64 - 10 {
            0 => DbId(10),
            id => DbId(id),
        };

        match random.next(20) {
            0 | 1 if *key == DbValue::None => {}
            0 => {
                let count = indexes.remove_index(view, alloc, keys, key).unwrap();
                let expected = model.remove(key).map_or(0, |values| {
                    values.values().map(|ids| ids.len() as u64).sum()
                });
                assert_eq!(count, expected);
            }
            1 if !model.contains_key(key) => {
                indexes.insert_index(view, alloc, keys, key).unwrap();
                model.insert(key.clone(), BTreeMap::new());
            }
            2..=10 => {
                let has = model
                    .get(key)
                    .is_some_and(|values| values.get(value).is_some_and(|ids| ids.contains(&id.0)));

                let taken = *key == DbValue::None
                    && model[key].get(value).is_some_and(|ids| !ids.is_empty());

                if taken {
                    let error = indexes.insert(view, alloc, key_id, value, id).unwrap_err();
                    let owner = model[key][value].first().unwrap();
                    assert_eq!(
                        error.description,
                        format!("Alias '{value}' is taken by '{owner}'")
                    );
                } else if !has {
                    indexes.insert(view, alloc, key_id, value, id).unwrap();

                    if let Some(values) = model.get_mut(key) {
                        values.entry(value.clone()).or_default().insert(id.0);
                    }
                }
            }
            _ => {
                indexes.remove(view, alloc, key_id, value, id).unwrap();

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
                    let mut keys = keys(view, alloc);
                    let mut indexes = Indexes::default();
                    let mut model = model();

                    for _ in 0..500 {
                        random_op(
                            view,
                            alloc,
                            &mut keys,
                            &mut indexes,
                            &mut model,
                            &mut random,
                        );
                        assert_indexes(view, &keys, &indexes, &model);
                    }

                    verify(alloc, view, &live(view, &keys, &indexes));
                });
            }
        }

        #[test]
        fn indexes_failure_should_fail() {
            let long = DbValue::from(LONG);
            assert_failures(
                |view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    keys.insert(view, alloc, &2.into()).unwrap();
                    (keys, Indexes::default())
                },
                |view, alloc, state| {
                    let (mut keys, mut indexes) = state;
                    let keys = &mut keys;
                    indexes.insert_index(view, alloc, keys, &long)?;
                    indexes.insert_index(view, alloc, keys, &1.into())?;
                    indexes.insert_index(view, alloc, keys, &2.into())?;
                    let key = indexes.index(&long).unwrap().key_id;

                    for id in 1..=3 {
                        indexes.insert(view, alloc, key, &long, DbId(id))?;
                    }

                    indexes.search(view, &long, &long)?;
                    indexes.insert(view, alloc, ALIAS, &long, DbId(1))?;
                    indexes.aliases().search(view, &long)?;
                    indexes.aliases().values(view)?;
                    indexes.index(&long).unwrap().values(view)?;
                    indexes.remove(view, alloc, ALIAS, &long, DbId(1))?;
                    Indexes::load(view, keys, &indexes.encode())?;

                    for id in 1..=3 {
                        indexes.remove(view, alloc, key, &long, DbId(id))?;
                    }

                    indexes.insert(view, alloc, key, &long, DbId(1))?;
                    indexes.insert(view, alloc, key, &long, DbId(2))?;
                    indexes.remove_index(view, alloc, keys, &1.into())?;
                    indexes.remove_index(view, alloc, keys, &long)?;
                    indexes.remove_index(view, alloc, keys, &2.into())?;
                    Ok(())
                },
            );
        }

        mod search {
            use super::*;

            #[test]
            fn search_with_colliding_hash_should_compare_value() {
                with_view(|view, alloc| {
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let mut indexes = Indexes::default();
                    indexes
                        .insert_index(view, alloc, &mut keys, &1.into())
                        .unwrap();
                    let key = indexes.list[0].key_id;
                    indexes
                        .insert(view, alloc, key, &"a".into(), DbId(1))
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
                        .remove(view, alloc, key, &"b".into(), DbId(2))
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
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let other = keys.insert(view, alloc, &2.into()).unwrap();
                    let mut indexes = Indexes::default();
                    indexes
                        .insert_index(view, alloc, &mut keys, &1.into())
                        .unwrap();
                    let key = indexes.list[0].key_id;
                    indexes
                        .insert(view, alloc, key, &1.into(), DbId(1))
                        .unwrap();
                    indexes
                        .remove(view, alloc, key, &1.into(), DbId(2))
                        .unwrap();
                    indexes
                        .insert(view, alloc, key, &2.into(), DbId(1))
                        .unwrap();
                    indexes
                        .insert(view, alloc, key, &2.into(), DbId(2))
                        .unwrap();
                    indexes
                        .remove(view, alloc, key, &2.into(), DbId(3))
                        .unwrap();
                    indexes
                        .remove(view, alloc, other, &2.into(), DbId(1))
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
