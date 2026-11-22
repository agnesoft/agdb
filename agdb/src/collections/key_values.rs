use crate::DbError;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::collections::graph::GraphHeader;
use crate::collections::value;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::alloc::u32_at;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;

// The values of an element are one allocation at the position its graph
// data holds, the last data field of a node and the only one of an edge:
// `[len u32][cap u32]` then `cap` pairs of the key and the value index. It
// is freed when its last value is removed.

const HEADER: u64 = 8;
const PAIR: u64 = 32;

struct List {
    pos: u64,
    len: u64,
    cap: u64,
}

type Pair = (DbValueIndex, DbValueIndex);

fn size(cap: u64) -> u64 {
    HEADER + cap * PAIR
}

/// The pairs an allocation for at least `cap` pairs holds.
fn fit(alloc: &Allocator, cap: u64) -> u64 {
    (alloc.usable(size(cap)) - HEADER) / PAIR
}

/// The next size class while the list is small, then double the pages.
fn grown(alloc: &Allocator, cap: u64) -> u64 {
    if alloc.usable(size(cap + 1)) < alloc.page_size() {
        fit(alloc, cap + 1)
    } else {
        fit(alloc, (cap * 2).max(cap + 1))
    }
}

fn pair_pos(list: &List, i: u64) -> u64 {
    list.pos + HEADER + i * PAIR
}

fn pair_bytes(pair: &Pair) -> [u8; PAIR as usize] {
    let mut bytes = [0_u8; PAIR as usize];
    bytes[..16].copy_from_slice(&pair.0.data());
    bytes[16..].copy_from_slice(&pair.1.data());
    bytes
}

fn pairs(view: &impl View, list: &List) -> Result<Vec<Pair>, DbError> {
    Ok(view
        .read(pair_pos(list, 0), list.len * PAIR)?
        .as_chunks::<32>()
        .0
        .iter()
        .map(|pair| {
            let mut key = [0_u8; 16];
            let mut value = [0_u8; 16];
            key.copy_from_slice(&pair[..16]);
            value.copy_from_slice(&pair[16..]);
            (DbValueIndex::from_data(key), DbValueIndex::from_data(value))
        })
        .collect())
}

fn find(view: &impl View, pairs: &[Pair], key: &DbValue) -> Result<Option<usize>, DbError> {
    let (kind, bytes) = value::encode(key);

    for (i, pair) in pairs.iter().enumerate() {
        if value::equals_bytes(view, &pair.0, kind, &bytes)? {
            return Ok(Some(i));
        }
    }

    Ok(None)
}

fn load(view: &impl View, pair: &Pair) -> Result<DbKeyValue, DbError> {
    Ok(DbKeyValue {
        key: value::load(view, &pair.0)?,
        value: value::load(view, &pair.1)?,
    })
}

fn write_header(view: &mut impl ViewMut, list: &List) -> Result<(), DbError> {
    let header = (list.len | (list.cap << 32)).to_le_bytes();
    view.write(list.pos, &header)
}

/// The values of the elements of the graph.
pub(crate) struct KeyValues<'a>(pub(crate) &'a GraphHeader);

impl KeyValues<'_> {
    fn list(&self, view: &impl View, id: DbId) -> Result<List, DbError> {
        let pos = if 0 < id.0 {
            self.0.node(view, id)?.map(|node| node.data[2])
        } else {
            self.0.edge(view, id)?.map(|edge| edge.data)
        }
        .unwrap_or(0);

        if pos == 0 {
            return Ok(List {
                pos,
                len: 0,
                cap: 0,
            });
        }

        let header = view.read(pos, HEADER)?;

        Ok(List {
            pos,
            len: u64::from(u32_at(&header, 0)),
            cap: u64::from(u32_at(&header, 4)),
        })
    }

    fn set_pos(&self, view: &mut impl ViewMut, id: DbId, pos: u64) -> Result<(), DbError> {
        let index = if 0 < id.0 { 2 } else { 0 };
        self.0.set_data(view, id, index, &[pos])
    }

    /// Moves the list to an allocation of `cap` pairs.
    fn resize(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &mut List,
        cap: u64,
    ) -> Result<(), DbError> {
        let pos = if list.pos == 0 {
            alloc.alloc(view, size(cap))?
        } else {
            alloc.realloc(view, list.pos, size(list.cap), size(cap))?
        };

        if pos != list.pos {
            self.set_pos(view, id, pos)?;
        }

        list.pos = pos;
        list.cap = cap;
        write_header(view, list)
    }

    fn push_to(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &mut List,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        if list.len == list.cap {
            self.resize(view, alloc, id, list, grown(alloc, list.cap))?;
        }

        let key = value::store(view, alloc, &key_value.key)?;
        let value = value::store(view, alloc, &key_value.value)?;
        view.write(pair_pos(list, list.len), &pair_bytes(&(key, value)))?;
        list.len += 1;
        write_header(view, list)
    }

    fn free_list(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        list: &List,
        pairs: &[Pair],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let mut removed = Vec::new();

        for pair in pairs {
            removed.push(load(view, pair)?);
            value::free(view, alloc, &pair.0)?;
            value::free(view, alloc, &pair.1)?;
        }

        alloc.free(view, list.pos, size(list.cap))?;
        self.set_pos(view, id, 0)?;
        Ok(removed)
    }

    pub(crate) fn key_count(&self, view: &impl View, id: DbId) -> Result<u64, DbError> {
        Ok(self.list(view, id)?.len)
    }

    pub(crate) fn keys(&self, view: &impl View, id: DbId) -> Result<Vec<DbValue>, DbError> {
        pairs(view, &self.list(view, id)?)?
            .iter()
            .map(|pair| value::load(view, &pair.0))
            .collect()
    }

    pub(crate) fn values(&self, view: &impl View, id: DbId) -> Result<Vec<DbKeyValue>, DbError> {
        pairs(view, &self.list(view, id)?)?
            .iter()
            .map(|pair| load(view, pair))
            .collect()
    }

    pub(crate) fn value(
        &self,
        view: &impl View,
        id: DbId,
        key: &DbValue,
    ) -> Result<Option<DbValue>, DbError> {
        let pairs = pairs(view, &self.list(view, id)?)?;

        find(view, &pairs, key)?
            .map(|i| value::load(view, &pairs[i].1))
            .transpose()
    }

    /// The values of the `keys` the element has, in the order of the `keys`.
    pub(crate) fn values_by_keys(
        &self,
        view: &impl View,
        id: DbId,
        keys: &[DbValue],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let pairs = pairs(view, &self.list(view, id)?)?;
        let mut values = Vec::new();

        for key in keys {
            if let Some(i) = find(view, &pairs, key)? {
                values.push(DbKeyValue {
                    key: key.clone(),
                    value: value::load(view, &pairs[i].1)?,
                });
            }
        }

        Ok(values)
    }

    /// Appends the `key_value` without looking for its key.
    pub(crate) fn push(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        let mut list = self.list(view, id)?;
        self.push_to(view, alloc, id, &mut list, key_value)
    }

    /// Sets the value of the key returning the value it replaced.
    pub(crate) fn insert(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<Option<DbKeyValue>, DbError> {
        let mut list = self.list(view, id)?;
        let pairs = pairs(view, &list)?;

        let Some(i) = find(view, &pairs, &key_value.key)? else {
            self.push_to(view, alloc, id, &mut list, key_value)?;
            return Ok(None);
        };

        let old = value::load(view, &pairs[i].1)?;
        value::free(view, alloc, &pairs[i].1)?;
        let new = value::store(view, alloc, &key_value.value)?;
        view.write(pair_pos(&list, i as u64) + 16, &new.data())?;

        Ok(Some(DbKeyValue {
            key: key_value.key.clone(),
            value: old,
        }))
    }

    /// Removes the key returning its value.
    pub(crate) fn remove(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        key: &DbValue,
    ) -> Result<Option<DbKeyValue>, DbError> {
        let mut list = self.list(view, id)?;
        let pairs = pairs(view, &list)?;

        let Some(i) = find(view, &pairs, key)? else {
            return Ok(None);
        };

        if list.len == 1 {
            return Ok(self.free_list(view, alloc, id, &list, &pairs)?.pop());
        }

        let removed = load(view, &pairs[i])?;
        value::free(view, alloc, &pairs[i].0)?;
        value::free(view, alloc, &pairs[i].1)?;
        let tail = pairs[i + 1..]
            .iter()
            .flat_map(pair_bytes)
            .collect::<Vec<u8>>();
        view.write(pair_pos(&list, i as u64), &tail)?;
        list.len -= 1;
        write_header(view, &list)?;
        Ok(Some(removed))
    }

    /// Removes all values of the element returning them.
    pub(crate) fn remove_all(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let list = self.list(view, id)?;

        if list.pos == 0 {
            return Ok(Vec::new());
        }

        let pairs = pairs(view, &list)?;
        self.free_list(view, alloc, id, &list, &pairs)
    }

    /// Makes room for `additional` values.
    pub(crate) fn reserve(
        &self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        id: DbId,
        additional: u64,
    ) -> Result<(), DbError> {
        let mut list = self.list(view, id)?;
        let cap = list.len + additional;

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

    type Model = BTreeMap<i64, Vec<DbKeyValue>>;

    fn keys_pool() -> Vec<DbValue> {
        vec![
            DbValue::from("k"),
            DbValue::from("a key longer than 15 bytes"),
            DbValue::from(1_u64),
            DbValue::from(vec![1_i64, 2]),
        ]
    }

    fn kv(key: &DbValue, value: &DbValue) -> DbKeyValue {
        DbKeyValue {
            key: key.clone(),
            value: value.clone(),
        }
    }

    /// The allocations of the values of the `ids`: their lists and the
    /// stored keys and values.
    fn live(view: &TestView, graph: &GraphHeader, ids: &[DbId]) -> Vec<(u64, u64)> {
        let mut live = Vec::new();

        for id in ids {
            let list = KeyValues(graph).list(view, *id).unwrap();

            live.extend((list.pos != 0).then(|| (list.pos, size(list.cap))));

            for pair in pairs(view, &list).unwrap() {
                live.extend(allocation(&pair.0));
                live.extend(allocation(&pair.1));
            }
        }

        live
    }

    fn assert_values(view: &TestView, graph: &GraphHeader, model: &Model) {
        for (id, expected) in model {
            let id = DbId(*id);
            assert_eq!(&KeyValues(graph).values(view, id).unwrap(), expected);
            assert_eq!(
                KeyValues(graph).key_count(view, id).unwrap(),
                expected.len() as u64
            );
            let expected_keys = expected.iter().map(|kv| kv.key.clone()).collect::<Vec<_>>();
            assert_eq!(KeyValues(graph).keys(view, id).unwrap(), expected_keys);

            for kv in expected {
                assert_eq!(
                    KeyValues(graph).value(view, id, &kv.key).unwrap().as_ref(),
                    Some(&kv.value)
                );
            }

            let mut reversed = keys_pool();
            reversed.reverse();
            let by_keys = reversed
                .iter()
                .filter_map(|key| expected.iter().find(|kv| kv.key == *key).cloned())
                .collect::<Vec<_>>();
            assert_eq!(
                KeyValues(graph)
                    .values_by_keys(view, id, &reversed)
                    .unwrap(),
                by_keys
            );
        }
    }

    fn random_op(
        view: &mut TestView,
        alloc: &mut Allocator,
        graph: &GraphHeader,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let keys = keys_pool();
        let values = all_values();
        let id = DbId(
            *model
                .keys()
                .nth(random.next(model.len() as u64) as usize)
                .unwrap(),
        );
        let key = &keys[random.next(keys.len() as u64) as usize];
        let value = &values[random.next(values.len() as u64) as usize];
        let expected = model.get_mut(&id.0).unwrap();
        let found = expected.iter().position(|kv| kv.key == *key);

        match random.next(10) {
            0..=4 => {
                let old = KeyValues(graph)
                    .insert(view, alloc, id, &kv(key, value))
                    .unwrap();
                assert_eq!(old.as_ref(), found.map(|i| &expected[i]));

                match found {
                    Some(i) => expected[i].value = value.clone(),
                    None => expected.push(kv(key, value)),
                }
            }
            5..=7 => {
                let removed = KeyValues(graph).remove(view, alloc, id, key).unwrap();
                assert_eq!(removed, found.map(|i| expected.remove(i)));
            }
            8 => {
                let additional = random.next(5);
                KeyValues(graph)
                    .reserve(view, alloc, id, additional)
                    .unwrap();
                assert!(
                    expected.len() as u64 + additional
                        <= KeyValues(graph).list(view, id).unwrap().cap
                );
            }
            _ => assert_eq!(
                KeyValues(graph).remove_all(view, alloc, id).unwrap(),
                std::mem::take(expected)
            ),
        }
    }

    mod grown {
        use super::*;

        #[test]
        fn grown_should_take_next_size_class_then_double_pages() {
            let alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
            let mut caps = vec![0];

            while caps.last() < Some(&500) {
                caps.push(grown(&alloc, *caps.last().unwrap()));
            }

            assert_eq!(
                caps,
                [0, 1, 2, 3, 4, 5, 6, 7, 10, 15, 31, 63, 127, 255, 511]
            );
        }
    }

    mod key_values {
        use super::*;

        #[test]
        fn key_values_should_hold_values_over_random_operations() {
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                    let mut model = Model::from([(node.0, vec![]), (edge.0, vec![])]);

                    for _ in 0..300 {
                        random_op(view, alloc, &graph, &mut model, &mut random);
                        assert_values(view, &graph, &model);
                    }

                    let mut live = live(view, &graph, &[node, edge]);
                    live.extend(graph_allocations(&graph));
                    verify(alloc, view, &live);
                });
            }
        }

        #[test]
        fn key_values_failure_should_fail() {
            let long = DbValue::from("a value longer than 15 bytes");
            assert_failures(
                |view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                    KeyValues(&graph)
                        .push(view, alloc, node, &kv(&long, &long))
                        .unwrap();
                    (graph, node, edge)
                },
                |view, alloc, state| {
                    let (graph, node, edge) = state;
                    KeyValues(&graph).push(view, alloc, edge, &kv(&long, &long))?;
                    KeyValues(&graph).reserve(view, alloc, node, 2)?;
                    KeyValues(&graph).push(view, alloc, node, &kv(&1.into(), &long))?;
                    KeyValues(&graph).push(view, alloc, node, &kv(&2.into(), &long))?;
                    KeyValues(&graph).push(view, alloc, node, &kv(&3.into(), &long))?;
                    KeyValues(&graph).insert(view, alloc, node, &kv(&long, &long))?;
                    KeyValues(&graph).insert(view, alloc, node, &kv(&4.into(), &long))?;
                    KeyValues(&graph).key_count(view, node)?;
                    KeyValues(&graph).keys(view, node)?;
                    KeyValues(&graph).values(view, node)?;
                    KeyValues(&graph).value(view, node, &long)?;
                    KeyValues(&graph).values_by_keys(view, node, &[long.clone(), 2.into()])?;
                    KeyValues(&graph).remove(view, alloc, node, &long)?;
                    KeyValues(&graph).remove_all(view, alloc, node)?;
                    KeyValues(&graph).push(view, alloc, node, &kv(&long, &long))?;
                    KeyValues(&graph).remove(view, alloc, node, &long)?;
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
                    let error = KeyValues(&graph)
                        .push(view, alloc, DbId(1), &kv(&1.into(), &1.into()))
                        .unwrap_err();
                    assert_eq!(error.description, "'1' is invalid index");
                });
            }

            #[test]
            fn push_should_keep_duplicate_keys() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    KeyValues(&graph)
                        .push(view, alloc, node, &kv(&1.into(), &1.into()))
                        .unwrap();
                    KeyValues(&graph)
                        .push(view, alloc, node, &kv(&1.into(), &2.into()))
                        .unwrap();
                    assert_eq!(
                        KeyValues(&graph).values(view, node).unwrap(),
                        vec![kv(&1.into(), &1.into()), kv(&1.into(), &2.into())]
                    );
                });
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn reserve_should_grow_list_in_place() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    KeyValues(&graph).reserve(view, alloc, node, 20).unwrap();
                    let list = KeyValues(&graph).list(view, node).unwrap();
                    KeyValues(&graph).reserve(view, alloc, node, 60).unwrap();
                    let reserved = KeyValues(&graph).list(view, node).unwrap();
                    assert_eq!((reserved.pos, reserved.cap), (list.pos, 63));
                });
            }
        }
    }
}
