use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::collections::graph::GraphHeader;
use crate::collections::keys::ALIAS;
use crate::collections::keys::KeyId;
use crate::collections::keys::Keys;
use crate::collections::value;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::alloc::u32_at;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;

// The values of an element are held by its graph data: 0 for none, a
// single label (a key whose value is `DbValue::None`) as `INLINE | key id`, or
// the position of one allocation `[used u32][cap u32]` followed by `cap`
// bytes of entries in insertion order. A label entry is
// `[key id | LABEL u32]`, any other `[key id u32][value DbValueIndex]`.
// Each entry holds a reference to its key in the [`Keys`], except the
// alias of a node, which is the value of [`ALIAS`]. The
// allocation is freed when its last value is removed.

const HEADER: u64 = 8;
const PAIR: u64 = 20;
const LABEL_SIZE: u64 = 4;
const LABEL: u32 = 1 << 31;
const INLINE: u64 = 1 << 63;

struct List {
    /// The graph data of the element.
    data: u64,
    used: u64,
    cap: u64,
}

impl List {
    fn pos(&self) -> u64 {
        if self.data & INLINE == 0 {
            self.data
        } else {
            0
        }
    }
}

struct Entry {
    key: KeyId,
    /// All zero for a label.
    value: DbValueIndex,
    /// The offset in the entries.
    at: u64,
}

fn is_label(value: &DbValueIndex) -> bool {
    *value == DbValueIndex::new()
}

fn entry_size(value: &DbValueIndex) -> u64 {
    if is_label(value) { LABEL_SIZE } else { PAIR }
}

fn push_entry(bytes: &mut Vec<u8>, key: KeyId, value: &DbValueIndex) {
    if is_label(value) {
        bytes.extend_from_slice(&(key.0 | LABEL).to_le_bytes());
    } else {
        bytes.extend_from_slice(&key.0.to_le_bytes());
        bytes.extend_from_slice(&value.data());
    }
}

fn entries_bytes(entries: &[Entry]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for entry in entries {
        push_entry(&mut bytes, entry.key, &entry.value);
    }

    bytes
}

/// The entry bytes an allocation for at least `bytes` of entries holds.
fn fit(alloc: &Allocator, bytes: u64) -> u64 {
    alloc.usable(HEADER + bytes) - HEADER
}

/// The next size class holding `needed` bytes while the list is small,
/// then double the pages.
fn grown(alloc: &Allocator, cap: u64, needed: u64) -> u64 {
    if alloc.usable(HEADER + needed) < alloc.page_size() {
        fit(alloc, needed)
    } else {
        fit(alloc, (cap * 2).max(needed))
    }
}

/// The entry at the offset `at` and the offset past it.
fn entry_at(bytes: &[u8], at: u64) -> Option<(Entry, u64)> {
    let word = u32_at(bytes.get(at as usize..(at + LABEL_SIZE) as usize)?, 0);
    let mut value = [0_u8; 16];
    let mut next = at + LABEL_SIZE;

    if word & LABEL == 0 {
        next = at + PAIR;
        value.copy_from_slice(bytes.get((at + LABEL_SIZE) as usize..next as usize)?);
    }

    let entry = Entry {
        key: KeyId(word & !LABEL),
        value: DbValueIndex::from_data(value),
        at,
    };

    Some((entry, next))
}

fn entries(view: &impl View, list: &List) -> Result<Vec<Entry>, DbError> {
    if list.data & INLINE != 0 {
        return Ok(vec![Entry {
            key: KeyId(list.data as u32),
            value: DbValueIndex::new(),
            at: 0,
        }]);
    }

    if list.data == 0 {
        return Ok(Vec::new());
    }

    let bytes = view.read(list.data + HEADER, list.used)?;
    let mut entries = Vec::new();
    let mut at = 0;

    while at < list.used {
        let Some((entry, next)) = entry_at(&bytes, at) else {
            return Err(DbError::db(
                DbErrorType::InvalidIndex,
                format!("Invalid values at '{}'", list.data),
            ));
        };

        entries.push(entry);
        at = next;
    }

    Ok(entries)
}

fn find(entries: &[Entry], key: KeyId) -> Option<usize> {
    entries.iter().position(|entry| entry.key == key)
}

fn write_header(view: &mut impl ViewMut, list: &List) -> Result<(), DbError> {
    let header = (list.used | (list.cap << 32)).to_le_bytes();
    view.write(list.data, &header)
}

/// The values of the elements of the graph, their keys being in the
/// [`Keys`].
pub(crate) struct KeyValues<'a>(pub(crate) &'a GraphHeader);

impl KeyValues<'_> {
    fn list(&self, view: &impl View, id: DbId) -> Result<List, DbError> {
        let data = if 0 < id.0 {
            self.0.node(view, id)?.map(|node| node.data)
        } else {
            self.0.edge(view, id)?.map(|edge| edge.data)
        }
        .unwrap_or(0);

        let mut list = List {
            data,
            used: 0,
            cap: 0,
        };

        if data & INLINE != 0 {
            list.used = LABEL_SIZE;
        } else if data != 0 {
            let header = view.read(data, HEADER)?;
            list.used = u64::from(u32_at(&header, 0));
            list.cap = u64::from(u32_at(&header, 4));
        }

        Ok(list)
    }

    fn set_data(&self, view: &mut impl ViewMut, id: DbId, data: u64) -> Result<(), DbError> {
        self.0.set_data(view, id, data)
    }

    /// Moves the entries to an allocation of `cap` bytes of entries.
    fn resize(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &mut List,
        cap: u64,
    ) -> Result<(), DbError> {
        let pos = if list.pos() == 0 {
            let pos = alloc.alloc(view, HEADER + cap)?;

            if list.data != 0 {
                view.write(pos + HEADER, &((list.data as u32) | LABEL).to_le_bytes())?;
            }

            pos
        } else {
            alloc.realloc(view, list.data, HEADER + list.cap, HEADER + cap)?
        };

        if pos != list.data {
            self.set_data(view, id, pos)?;
        }

        list.data = pos;
        list.cap = cap;
        write_header(view, list)
    }

    /// Writes the entry `bytes` at the offset `at`, the entries past them
    /// being dropped.
    fn write_at(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &mut List,
        at: u64,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        let used = at + bytes.len() as u64;

        if list.cap < used {
            self.resize(view, alloc, id, list, grown(alloc, list.cap, used))?;
        }

        view.write(list.data + HEADER + at, bytes)?;
        list.used = used;
        write_header(view, list)
    }

    /// Appends the entry of the `key`, to which the caller added the
    /// reference of the entry.
    fn push_to(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &mut List,
        key: KeyId,
        value: &DbValue,
    ) -> Result<(), DbError> {
        let value = value::store(view, alloc, value)?;

        if list.data == 0 && is_label(&value) {
            return self.set_data(view, id, INLINE | u64::from(key.0));
        }

        let mut bytes = Vec::new();
        push_entry(&mut bytes, key, &value);
        self.write_at(view, alloc, id, list, list.used, &bytes)
    }

    /// Loads and frees the value of the `entry`, releasing its key.
    fn free_entry(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        entry: &Entry,
    ) -> Result<DbValue, DbError> {
        let value = value::load(view, &entry.value)?;
        value::free(view, alloc, &entry.value)?;

        if entry.key != ALIAS {
            keys.remove(view, alloc, entry.key)?;
        }

        Ok(value)
    }

    /// The number of keys, the alias not counted.
    pub(crate) fn key_count(&self, view: &impl View, id: DbId) -> Result<u64, DbError> {
        Ok(self.entries(view, id)?.len() as u64)
    }

    /// The entries without the alias.
    fn entries(&self, view: &impl View, id: DbId) -> Result<Vec<Entry>, DbError> {
        let mut entries = entries(view, &self.list(view, id)?)?;
        entries.retain(|entry| entry.key != ALIAS);
        Ok(entries)
    }

    pub(crate) fn keys(
        &self,
        view: &impl View,
        keys: &Keys,
        id: DbId,
    ) -> Result<Vec<DbValue>, DbError> {
        self.entries(view, id)?
            .iter()
            .map(|entry| keys.key(view, entry.key))
            .collect()
    }

    pub(crate) fn values(
        &self,
        view: &impl View,
        keys: &Keys,
        id: DbId,
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let mut values = Vec::new();

        for entry in self.entries(view, id)? {
            values.push(DbKeyValue {
                key: keys.key(view, entry.key)?,
                value: value::load(view, &entry.value)?,
            });
        }

        Ok(values)
    }

    /// The value of the `key`, `DbValue::None` for a label and the alias
    /// for [`ALIAS`].
    pub(crate) fn value(
        &self,
        view: &impl View,
        id: DbId,
        key: KeyId,
    ) -> Result<Option<DbValue>, DbError> {
        let entries = entries(view, &self.list(view, id)?)?;

        find(&entries, key)
            .map(|i| value::load(view, &entries[i].value))
            .transpose()
    }

    /// The values of the `keys`, `None` for those the element does not
    /// have.
    pub(crate) fn values_by_keys(
        &self,
        view: &impl View,
        id: DbId,
        keys: &[KeyId],
    ) -> Result<Vec<Option<DbValue>>, DbError> {
        let entries = entries(view, &self.list(view, id)?)?;

        keys.iter()
            .map(|key| {
                find(&entries, *key)
                    .map(|i| value::load(view, &entries[i].value))
                    .transpose()
            })
            .collect()
    }

    /// Appends the `key_value` without looking for its key, returning the
    /// id of the key.
    pub(crate) fn push(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<KeyId, DbError> {
        let mut list = self.list(view, id)?;
        let key = keys.insert(view, alloc, &key_value.key)?;
        self.push_to(view, alloc, id, &mut list, key, &key_value.value)?;
        Ok(key)
    }

    /// Sets the value of the key returning the id of the key and the
    /// value it replaced.
    pub(crate) fn insert(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(KeyId, Option<DbValue>), DbError> {
        if let Some(key) = keys.id(view, &key_value.key)? {
            let old = self.set(view, alloc, keys, id, key, &key_value.value)?;
            return Ok((key, old));
        }

        let mut list = self.list(view, id)?;
        let key = keys.insert(view, alloc, &key_value.key)?;
        self.push_to(view, alloc, id, &mut list, key, &key_value.value)?;
        Ok((key, None))
    }

    /// Sets the value of the existing `key`, or the alias for
    /// [`ALIAS`], returning the value it replaced.
    pub(crate) fn set(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        id: DbId,
        key: KeyId,
        value: &DbValue,
    ) -> Result<Option<DbValue>, DbError> {
        let mut list = self.list(view, id)?;
        let mut entries = entries(view, &list)?;

        let Some(i) = find(&entries, key) else {
            if key != ALIAS {
                keys.acquire(view, key)?;
            }

            self.push_to(view, alloc, id, &mut list, key, value)?;
            return Ok(None);
        };

        let old = value::load(view, &entries[i].value)?;
        value::free(view, alloc, &entries[i].value)?;
        let new = value::store(view, alloc, value)?;
        let at = entries[i].at;

        if entry_size(&new) == entry_size(&entries[i].value) {
            if !is_label(&new) {
                view.write(list.data + HEADER + at + 4, &new.data())?;
            }
        } else {
            entries[i].value = new;
            let bytes = entries_bytes(&entries[i..]);
            self.write_at(view, alloc, id, &mut list, at, &bytes)?;
        }

        Ok(Some(old))
    }

    /// Removes the key returning its value.
    pub(crate) fn remove(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        id: DbId,
        key: KeyId,
    ) -> Result<Option<DbValue>, DbError> {
        let mut list = self.list(view, id)?;
        let entries = entries(view, &list)?;

        let Some(i) = find(&entries, key) else {
            return Ok(None);
        };

        if entries.len() == 1 {
            return Ok(self
                .remove_all(view, alloc, keys, id)?
                .pop()
                .map(|entry| entry.1));
        }

        let removed = self.free_entry(view, alloc, keys, &entries[i])?;
        let bytes = entries_bytes(&entries[i + 1..]);
        self.write_at(view, alloc, id, &mut list, entries[i].at, &bytes)?;
        Ok(Some(removed))
    }

    /// Removes all values of the element, the alias included, returning
    /// them with the ids of their keys, which are freed unless referenced
    /// elsewhere.
    pub(crate) fn remove_all(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        keys: &mut Keys,
        id: DbId,
    ) -> Result<Vec<(KeyId, DbValue)>, DbError> {
        let list = self.list(view, id)?;

        if list.data == 0 {
            return Ok(Vec::new());
        }

        let mut removed = Vec::new();

        for entry in entries(view, &list)? {
            removed.push((entry.key, self.free_entry(view, alloc, keys, &entry)?));
        }

        if list.pos() != 0 {
            alloc.free(view, list.data, HEADER + list.cap)?;
        }

        self.set_data(view, id, 0)?;
        Ok(removed)
    }

    /// Makes room for `additional` values that are not labels.
    pub(crate) fn reserve(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        additional: u64,
    ) -> Result<(), DbError> {
        let mut list = self.list(view, id)?;
        let cap = list.used + additional * PAIR;

        if list.cap < cap {
            self.resize(view, alloc, id, &mut list, fit(alloc, cap))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::graph::tests::allocations as graph_allocations;
    use crate::collections::keys::tests::allocations as keys_allocations;
    use crate::collections::keys::tests::refs;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::allocation;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::values as all_values;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::tests::XorShift;
    use crate::storage::root::Layout;
    use std::collections::BTreeMap;

    /// The values of the elements, the alias under the key `DbValue::None`.
    type Model = BTreeMap<i64, Vec<DbKeyValue>>;

    const LONG: &str = "a value longer than 15 bytes";

    fn keys_pool() -> Vec<DbValue> {
        vec![
            DbValue::from("k"),
            DbValue::from("a key longer than 15 bytes"),
            DbValue::from(1_u64),
            DbValue::from(vec![1_i64, 2]),
            DbValue::None,
        ]
    }

    fn key_id(view: &TestView, keys: &Keys, key: &DbValue) -> Option<KeyId> {
        if *key == DbValue::None {
            return Some(ALIAS);
        }

        keys.id(view, key).unwrap()
    }

    fn kv(key: &DbValue, value: &DbValue) -> DbKeyValue {
        DbKeyValue {
            key: key.clone(),
            value: value.clone(),
        }
    }

    /// A graph of a node and an edge from it to itself.
    fn graph(view: &mut TestView, alloc: &mut Allocator) -> (GraphHeader, DbId, DbId) {
        let mut graph = GraphHeader::new(Layout::default());
        let node = graph.insert_node(view, alloc).unwrap();
        let edge = graph.insert_edge(view, alloc, node, node).unwrap();
        (graph, node, edge)
    }

    /// The allocations of the graph, the keys and the values of the
    /// `ids`.
    fn live(view: &TestView, graph: &GraphHeader, keys: &Keys, ids: &[DbId]) -> Vec<(u64, u64)> {
        let mut live = graph_allocations(graph);
        live.extend(keys_allocations(keys, view));

        for id in ids {
            let list = KeyValues(graph).list(view, *id).unwrap();
            live.extend((list.pos() != 0).then_some((list.data, HEADER + list.cap)));

            for entry in entries(view, &list).unwrap() {
                live.extend(allocation(&entry.value));
            }
        }

        live
    }

    fn assert_values(view: &TestView, graph: &GraphHeader, keys: &Keys, model: &Model) {
        let mut counts = BTreeMap::<DbValue, u64>::new();

        for (id, all) in model {
            let id = DbId(*id);
            let values = KeyValues(graph);
            let (alias, expected): (Vec<_>, Vec<_>) =
                all.iter().cloned().partition(|kv| kv.key == DbValue::None);
            let alias = alias.first().map(|kv| kv.value.clone());
            assert_eq!(values.value(view, id, ALIAS).unwrap(), alias);
            assert_eq!(values.values(view, keys, id).unwrap(), expected);
            assert_eq!(values.key_count(view, id).unwrap(), expected.len() as u64);
            let expected_keys = expected.iter().map(|kv| kv.key.clone()).collect::<Vec<_>>();
            assert_eq!(values.keys(view, keys, id).unwrap(), expected_keys);
            let mut pool = keys_pool();
            pool.reverse();
            let ids = pool
                .iter()
                .filter(|key| **key != DbValue::None)
                .filter_map(|key| keys.id(view, key).unwrap())
                .collect::<Vec<_>>();
            let by_keys = ids
                .iter()
                .map(|key| {
                    let key = keys.key(view, *key).unwrap();
                    expected
                        .iter()
                        .find(|kv| kv.key == key)
                        .map(|kv| kv.value.clone())
                })
                .collect::<Vec<_>>();
            assert_eq!(values.values_by_keys(view, id, &ids).unwrap(), by_keys);

            for kv in &expected {
                let key = keys.id(view, &kv.key).unwrap().unwrap();
                assert_eq!(values.value(view, id, key).unwrap(), Some(kv.value.clone()));
                *counts.entry(kv.key.clone()).or_default() += 1;
            }
        }

        assert_eq!(keys.len(), counts.len() as u64);

        for (key, count) in counts {
            assert_eq!(
                refs(keys, view, keys.id(view, &key).unwrap().unwrap()),
                count
            );
        }
    }

    fn random_op(
        view: &mut TestView,
        alloc: &mut Allocator,
        graph: &GraphHeader,
        keys: &mut Keys,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let pool = keys_pool();
        let values = all_values();
        let id = DbId(
            *model
                .keys()
                .nth(random.next(model.len() as u64) as usize)
                .unwrap(),
        );
        let key = &pool[random.next(pool.len() as u64) as usize];
        let value = &values[random.next(values.len() as u64) as usize];
        let expected = model.get_mut(&id.0).unwrap();
        let found = expected.iter().position(|kv| kv.key == *key);

        match random.next(10) {
            0..=4 => {
                let old = if *key == DbValue::None {
                    KeyValues(graph)
                        .set(view, alloc, keys, id, ALIAS, value)
                        .unwrap()
                } else {
                    let (key_id, old) = KeyValues(graph)
                        .insert(view, alloc, keys, id, &kv(key, value))
                        .unwrap();
                    assert_eq!(keys.key(view, key_id).unwrap(), *key);
                    old
                };
                assert_eq!(old.as_ref(), found.map(|i| &expected[i].value));

                match found {
                    Some(i) => expected[i].value = value.clone(),
                    None => expected.push(kv(key, value)),
                }
            }
            5..=7 => {
                let removed = key_id(view, keys, key)
                    .map(|key| KeyValues(graph).remove(view, alloc, keys, id, key).unwrap());
                assert_eq!(removed.flatten(), found.map(|i| expected.remove(i).value));
            }
            8 => {
                let additional = random.next(5);
                let used = KeyValues(graph).list(view, id).unwrap().used;
                KeyValues(graph)
                    .reserve(view, alloc, id, additional)
                    .unwrap();
                assert!(used + additional * PAIR <= KeyValues(graph).list(view, id).unwrap().cap);
            }
            _ => {
                let removed = KeyValues(graph)
                    .remove_all(view, alloc, keys, id)
                    .unwrap()
                    .into_iter()
                    .map(|pair| pair.1)
                    .collect::<Vec<_>>();
                let expected = std::mem::take(expected)
                    .into_iter()
                    .map(|kv| kv.value)
                    .collect::<Vec<_>>();
                assert_eq!(removed, expected);
            }
        }
    }

    mod grown {
        use super::*;

        #[test]
        fn grown_should_take_next_size_class_then_double_pages() {
            let alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
            let mut caps = vec![0];

            while caps.last() < Some(&10_000) {
                let cap = *caps.last().unwrap();
                caps.push(grown(&alloc, cap, cap + PAIR));
            }

            assert_eq!(
                caps,
                [
                    0, 24, 56, 88, 120, 160, 192, 240, 328, 496, 1016, 2040, 4088, 8184, 16376
                ]
            );
        }
    }

    mod entries {
        use super::*;

        #[test]
        fn entries_of_truncated_list_should_fail() {
            with_view(|view, alloc| {
                let (graph, node, _) = graph(view, alloc);
                let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                let values = KeyValues(&graph);
                values
                    .push(view, alloc, &mut keys, node, &kv(&1.into(), &1.into()))
                    .unwrap();
                let mut list = values.list(view, node).unwrap();

                for used in [2, 8] {
                    list.used = used;
                    let error = entries(view, &list).err().unwrap();
                    assert_eq!(
                        error.description,
                        format!("Invalid values at '{}'", list.data)
                    );
                }
            });
        }
    }

    mod key_values {
        use super::*;

        #[test]
        fn key_values_should_hold_values_over_random_operations() {
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let mut model = Model::from([(node.0, vec![]), (edge.0, vec![])]);

                    for _ in 0..300 {
                        random_op(view, alloc, &graph, &mut keys, &mut model, &mut random);
                        assert_values(view, &graph, &keys, &model);
                    }

                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }
        }

        #[test]
        fn key_values_failure_should_fail() {
            let long = DbValue::from(LONG);
            assert_failures(
                |view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    KeyValues(&graph)
                        .push(view, alloc, &mut keys, node, &kv(&long, &long))
                        .unwrap();
                    (graph, keys, node, edge)
                },
                |view, alloc, state| {
                    let (graph, mut keys, node, edge) = state;
                    let values = KeyValues(&graph);
                    let keys = &mut keys;
                    values.push(view, alloc, keys, edge, &kv(&5.into(), &DbValue::None))?;
                    values.push(view, alloc, keys, edge, &kv(&long, &long))?;
                    values.reserve(view, alloc, node, 2)?;
                    values.push(view, alloc, keys, node, &kv(&1.into(), &long))?;
                    values.push(view, alloc, keys, node, &kv(&2.into(), &long))?;
                    values.push(view, alloc, keys, node, &kv(&3.into(), &long))?;
                    let (key, _) = values.insert(view, alloc, keys, node, &kv(&long, &long))?;
                    values.insert(view, alloc, keys, node, &kv(&4.into(), &long))?;
                    values.insert(view, alloc, keys, edge, &kv(&4.into(), &long))?;
                    values.insert(view, alloc, keys, node, &kv(&1.into(), &DbValue::None))?;
                    values.insert(view, alloc, keys, node, &kv(&1.into(), &long))?;
                    values.remove(view, alloc, keys, edge, KeyId(4))?;
                    values.set(view, alloc, keys, node, ALIAS, &long)?;
                    values.set(view, alloc, keys, node, ALIAS, &"a".into())?;
                    values.set(view, alloc, keys, edge, ALIAS, &long)?;
                    values.remove(view, alloc, keys, edge, ALIAS)?;
                    values.key_count(view, node)?;
                    values.keys(view, keys, node)?;
                    values.values(view, keys, node)?;
                    values.value(view, node, key)?;
                    values.values_by_keys(view, node, &[key, KeyId(2)])?;
                    values.remove(view, alloc, keys, node, key)?;
                    values.remove_all(view, alloc, keys, node)?;
                    values.push(view, alloc, keys, node, &kv(&long, &long))?;
                    values.remove(view, alloc, keys, node, key)?;
                    Ok(())
                },
            );
        }

        mod push {
            use super::*;

            #[test]
            fn push_to_missing_element_should_fail() {
                with_view(|view, alloc| {
                    let graph = GraphHeader::new(Layout::default());
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let error = KeyValues(&graph)
                        .push(view, alloc, &mut keys, DbId(1), &kv(&1.into(), &1.into()))
                        .unwrap_err();
                    assert_eq!(error.description, "'1' is invalid index");
                });
            }

            #[test]
            fn push_label_to_empty_element_should_keep_it_inline() {
                with_view(|view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let label = kv(&LONG.into(), &DbValue::None);
                    let key = values.push(view, alloc, &mut keys, edge, &label).unwrap();
                    assert_eq!(
                        values.list(view, edge).unwrap().data,
                        INLINE | u64::from(key.0)
                    );
                    assert_eq!(
                        values.values(view, &keys, edge).unwrap(),
                        vec![label.clone()]
                    );
                    assert_eq!(values.value(view, edge, key).unwrap(), Some(DbValue::None));
                    assert_eq!(values.key_count(view, edge).unwrap(), 1);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                    let pair = kv(&1.into(), &LONG.into());
                    values.push(view, alloc, &mut keys, edge, &pair).unwrap();
                    assert_eq!(values.values(view, &keys, edge).unwrap(), vec![label, pair]);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }

            #[test]
            fn push_should_keep_duplicate_keys() {
                with_view(|view, alloc| {
                    let (graph, node, _) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let first = values
                        .push(view, alloc, &mut keys, node, &kv(&1.into(), &1.into()))
                        .unwrap();
                    let second = values
                        .push(view, alloc, &mut keys, node, &kv(&1.into(), &2.into()))
                        .unwrap();
                    assert_eq!(first, second);
                    assert_eq!(refs(&keys, view, first), 2);
                    assert_eq!(
                        values.values(view, &keys, node).unwrap(),
                        vec![kv(&1.into(), &1.into()), kv(&1.into(), &2.into())]
                    );
                });
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_changing_label_and_value_should_keep_order() {
                with_view(|view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let mut expected = vec![
                        kv(&1.into(), &1.into()),
                        kv(&2.into(), &DbValue::None),
                        kv(&3.into(), &LONG.into()),
                    ];

                    for pair in &expected {
                        values.push(view, alloc, &mut keys, node, pair).unwrap();
                    }

                    for (i, value) in [(1, LONG.into()), (0, DbValue::None), (2, DbValue::None)] {
                        let old = values
                            .insert(view, alloc, &mut keys, node, &kv(&expected[i].key, &value))
                            .unwrap()
                            .1;
                        assert_eq!(old, Some(std::mem::replace(&mut expected[i].value, value)));
                        assert_eq!(values.values(view, &keys, node).unwrap(), expected);
                    }

                    assert_eq!(values.list(view, node).unwrap().used, 4 + 20 + 4);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }
        }

        mod set {
            use super::*;

            #[test]
            fn set_alias_should_keep_it_apart_from_keys() {
                with_view(|view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let pair = kv(&1.into(), &LONG.into());
                    let alias = DbValue::from(LONG);
                    values.push(view, alloc, &mut keys, node, &pair).unwrap();
                    let old = values
                        .set(view, alloc, &mut keys, node, ALIAS, &alias)
                        .unwrap();
                    assert_eq!(old, None);
                    assert_eq!(values.values(view, &keys, node).unwrap(), vec![pair]);
                    assert_eq!(values.keys(view, &keys, node).unwrap(), vec![1.into()]);
                    assert_eq!(values.key_count(view, node).unwrap(), 1);
                    let value = values.value(view, node, ALIAS).unwrap();
                    assert_eq!(value, Some(alias.clone()));
                    assert_eq!(keys.len(), 1);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                    let removed = values.remove_all(view, alloc, &mut keys, node).unwrap();
                    assert_eq!(removed, [(KeyId(1), LONG.into()), (ALIAS, alias)]);
                    assert_eq!(keys.len(), 0);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_last_reference_should_free_key() {
                with_view(|view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let key = values
                        .push(view, alloc, &mut keys, node, &kv(&LONG.into(), &1.into()))
                        .unwrap();
                    values
                        .push(view, alloc, &mut keys, edge, &kv(&LONG.into(), &2.into()))
                        .unwrap();
                    values.remove(view, alloc, &mut keys, node, key).unwrap();
                    assert_eq!(keys.id(view, &LONG.into()).unwrap(), Some(key));
                    values.remove(view, alloc, &mut keys, edge, key).unwrap();
                    assert_eq!(keys.id(view, &LONG.into()).unwrap(), None);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn reserve_with_inline_label_should_move_it_to_list() {
                with_view(|view, alloc| {
                    let (graph, node, edge) = graph(view, alloc);
                    let mut keys = Keys::new(DEFAULT_PAGE_SHIFT);
                    let values = KeyValues(&graph);
                    let label = kv(&1.into(), &DbValue::None);
                    values.push(view, alloc, &mut keys, node, &label).unwrap();
                    values.reserve(view, alloc, node, 1).unwrap();
                    let list = values.list(view, node).unwrap();
                    assert_eq!((list.pos(), list.used, list.cap), (list.data, 4, 24));
                    assert_eq!(values.values(view, &keys, node).unwrap(), vec![label]);
                    verify(alloc, view, &live(view, &graph, &keys, &[node, edge]));
                });
            }

            #[test]
            fn reserve_should_grow_list_in_place() {
                with_view(|view, alloc| {
                    let (graph, node, _) = graph(view, alloc);
                    KeyValues(&graph).reserve(view, alloc, node, 60).unwrap();
                    let list = KeyValues(&graph).list(view, node).unwrap();
                    KeyValues(&graph).reserve(view, alloc, node, 150).unwrap();
                    let reserved = KeyValues(&graph).list(view, node).unwrap();
                    assert_eq!((reserved.data, reserved.cap), (list.data, 3064));
                });
            }
        }
    }
}
