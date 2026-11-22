//! The records format of agdb 0.17 and older (storage versions 0 and 1,
//! database versions 0 and 1), read only to migrate it. It is frozen: it
//! reads what those versions wrote whatever the current format becomes.

use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::collections::graph::Edge;
use crate::collections::graph::Node;
use crate::collections::value::decode;
use crate::db::db_value::STRING_META_VALUE;
use crate::db::db_value::VEC_F64_META_VALUE;
use crate::db::db_value::VEC_I64_META_VALUE;
use crate::db::db_value::VEC_U64_META_VALUE;
use crate::db::rebuild::Element;
use crate::db::rebuild::Source;
use crate::storage::StorageSlice;
use crate::storage::pages::View;
use crate::utilities::serialize::u64_at;
use std::cmp::Reverse;
use std::collections::HashMap;

// A file is records `[index u64][size u64][value]` tiling it, after the
// version record `[0][8][storage version]` unless it is of the storage
// version 0. A record of index 0 is free space. The record 1 is the root:
// `[graph][aliases][aliases by id][indexes][values]` of record indexes,
// after `[1 u64]` in the database version 1. A vector is
// `[len u64][elements]` and a multimap `[len][states][keys][values]` of
// vectors whose valid slots have the state 1.

const HEADER: u64 = 16;
/// The first record after the version record.
const FIRST: u64 = 24;
const ROOT: u64 = 1;
/// The size of the root of the database version 0.
const ROOT_V0: u64 = 40;
const VALID: u8 = 1;
const WORD: u64 = 8;
/// `[type u4][inline size u4]` in its last byte, the value or the index of
/// its record in the others.
const VALUE: u64 = 16;
const KEY_VALUE: u64 = 2 * VALUE;
/// `[key value][multimap index]`
const INDEX: u64 = VALUE + WORD;

/// The record indexes of a non-empty database.
struct Fields {
    /// The vectors `from`, `to`, `from_meta` and `to_meta` of the graph.
    columns: Vec<u64>,
    /// The length of the columns, the slot 0 included.
    slots: u64,
    aliases: u64,
    indexes: u64,
    values: u64,
}

/// The index, value position and size of a record.
type Record = (u64, u64, u64);
/// The key and value bytes of a multimap entry.
type Entry = (Vec<u8>, Vec<u8>);

pub(crate) struct Legacy<'a, V> {
    view: &'a V,
    /// The index, value position and size of the records sorted by index.
    records: Vec<Record>,
    /// The database version, 0 for agdb 0.10.0 and older.
    version: u64,
    fields: Option<Fields>,
}

/// The `len * elem` bytes after the `[len u64]` of `bytes`.
fn prefixed(bytes: &[u8], elem: u64) -> Result<&[u8], DbError> {
    let payload = bytes.get(..8).and_then(|len| {
        let end = u64_at(len, 0).checked_mul(elem)?.checked_add(WORD)?;
        bytes.get(8..end as usize)
    });

    payload.ok_or_else(|| {
        DbError::storage(
            DbErrorType::OutOfBounds,
            format!("Invalid vector length ({} bytes)", bytes.len()),
        )
    })
}

/// The element of the raw fields of a graph slot: a node has `from_meta`
/// and `from` not negative, an edge a negative `from`.
fn element(from: i64, to: i64, from_meta: i64, to_meta: i64) -> Element {
    if from_meta < 0 {
        return (None, None);
    }

    if from < 0 {
        return (
            None,
            Some(Edge {
                from: DbId(from.wrapping_neg()),
                to: DbId(to.wrapping_neg()),
                next_from: DbId(from_meta.wrapping_neg()),
                next_to: DbId(to_meta.wrapping_neg()),
                previous_from: DbId(0),
                previous_to: DbId(0),
                data: 0,
            }),
        );
    }

    (
        Some(Node {
            first_from: DbId(from.wrapping_neg()),
            first_to: DbId(to.wrapping_neg()),
            from_count: 0,
            to_count: 0,
            data: 0,
        }),
        None,
    )
}

impl<'a, V: View> Legacy<'a, V> {
    pub(crate) fn open(view: &'a V) -> Result<Self, DbError> {
        let (storage, records) = Self::scan(view)?;
        let mut legacy = Self {
            view,
            records,
            version: 1,
            fields: None,
        };

        let Ok((_, size)) = legacy.record(ROOT) else {
            return Ok(legacy);
        };

        legacy.version = 0;

        if size != ROOT_V0 {
            legacy.version = legacy.word(ROOT, 0)?;

            if legacy.version != 1 {
                return Err(DbError::db(
                    DbErrorType::NotAllowed,
                    format!(
                        "Database version '{}' is not supported by storage version '{storage}'",
                        legacy.version
                    ),
                ));
            }
        }

        let root = legacy.read(ROOT, legacy.version * WORD, 5 * WORD)?;
        let graph = legacy.read(u64_at(&root, 0), 0, 4 * WORD)?;
        let columns: Vec<u64> = (0..4).map(|column| u64_at(&graph, column)).collect();
        legacy.fields = Some(Fields {
            slots: legacy.columns(&columns)?,
            columns,
            aliases: u64_at(&root, 1),
            indexes: u64_at(&root, 3),
            values: u64_at(&root, 4),
        });

        Ok(legacy)
    }

    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    /// The storage version and the records of the `view`, of which the
    /// later one of an index wins.
    fn scan(view: &V) -> Result<(u64, Vec<Record>), DbError> {
        let len = view.len();
        let mut pos = 0;
        let mut storage = 0;

        let header = view.read(0, HEADER.min(len))?;

        if HEADER <= len && u64_at(&header, 0) == 0 {
            let size = u64_at(&header, 1);

            if size < WORD {
                return Err(DbError::storage(
                    DbErrorType::NotEnoughData,
                    format!("Invalid version record size ({size} < 8)"),
                ));
            }

            storage = u64_at(&view.read(HEADER, WORD)?, 0);
            pos = FIRST;
        }

        let mut records = vec![];

        while pos < len {
            let header = view.read(pos, HEADER)?;
            let index = u64_at(&header, 0);
            let size = u64_at(&header, 1);
            let remaining = len - pos - HEADER;

            if remaining < size {
                return Err(DbError::storage(
                    DbErrorType::OutOfBounds,
                    format!("Invalid record size ({size}) exceeds remaining storage ({remaining})"),
                ));
            }

            if index != 0 {
                records.push((index, pos + HEADER, size));
            }

            pos += HEADER + size;
        }

        records.sort_unstable_by_key(|record| (record.0, Reverse(record.1)));
        records.dedup_by_key(|record| record.0);
        Ok((storage, records))
    }

    /// The length of the graph `columns`, the same for each.
    fn columns(&self, columns: &[u64]) -> Result<u64, DbError> {
        let slots = self.word(columns[0], 0)?;

        for column in columns {
            let len = self.word(*column, 0)?;

            if len != slots {
                return Err(DbError::graph(
                    DbErrorType::InvalidIndex,
                    format!("Graph v1 vectors have different lengths ({len} != {slots})"),
                ));
            }
        }

        if slots == 0 {
            return Err(DbError::graph(
                DbErrorType::InvalidIndex,
                "Graph v1 vectors are empty",
            ));
        }

        Ok(slots)
    }

    /// The value position and size of the record `index`.
    fn record(&self, index: u64) -> Result<(u64, u64), DbError> {
        let found = self
            .records
            .binary_search_by_key(&index, |record| record.0)
            .map_err(|_| {
                DbError::storage(DbErrorType::NotFound, format!("Index ({index}) not found"))
            })?;
        let record = self.records[found];
        Ok((record.1, record.2))
    }

    /// The `len` bytes at `offset` of the value of the record `index`.
    fn read(&self, index: u64, offset: u64, len: u64) -> Result<StorageSlice<'a>, DbError> {
        let (pos, size) = self.record(index)?;

        match offset.checked_add(len) {
            Some(end) if end <= size => self.view.read(pos + offset, len),
            _ => Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!(
                    "Value size ({}) out of bounds ({size})",
                    offset.saturating_add(len)
                ),
            )),
        }
    }

    fn word(&self, index: u64, offset: u64) -> Result<u64, DbError> {
        Ok(u64_at(&self.read(index, offset, WORD)?, 0))
    }

    /// The whole value of the record `index`.
    fn bytes(&self, index: u64) -> Result<StorageSlice<'a>, DbError> {
        self.read(index, 0, self.record(index)?.1)
    }

    /// The `count` elements of `elem` bytes from `start` of the vector
    /// `index`.
    fn elements(
        &self,
        index: u64,
        start: u64,
        count: u64,
        elem: u64,
    ) -> Result<StorageSlice<'a>, DbError> {
        self.read(
            index,
            WORD.saturating_add(start.saturating_mul(elem)),
            count.saturating_mul(elem),
        )
    }

    /// The keys and values of `key` and `value` bytes of the valid slots
    /// among the `count` from `start` of the multimap `index`.
    fn map(
        &self,
        index: u64,
        start: u64,
        count: u64,
        key: u64,
        value: u64,
    ) -> Result<Vec<Entry>, DbError> {
        let header = self.read(index, WORD, 3 * WORD)?;
        let len = self.word(u64_at(&header, 0), 0)?;
        let start = start.min(len);
        let count = count.min(len - start);
        let states = self.elements(u64_at(&header, 0), start, count, 1)?;
        let keys = self.elements(u64_at(&header, 1), start, count, key)?;
        let values = self.elements(u64_at(&header, 2), start, count, value)?;
        let (key, value) = (key as usize, value as usize);

        Ok(states
            .iter()
            .enumerate()
            .filter(|state| *state.1 == VALID)
            .map(|state| {
                let slot = state.0;
                (
                    keys[slot * key..][..key].to_vec(),
                    values[slot * value..][..value].to_vec(),
                )
            })
            .collect())
    }

    /// Whether the `id` is a node and whether it is an edge.
    fn kind(&self, id: DbId) -> Result<(bool, bool), DbError> {
        let element = self.graph(id.0.unsigned_abs(), 1)?.remove(0);
        Ok((
            0 < id.0 && element.0.is_some(),
            id.0 < 0 && element.1.is_some(),
        ))
    }

    /// The string of the record `[len u64][utf8]` of `index`.
    fn string(&self, index: u64) -> Result<String, DbError> {
        Ok(String::from_utf8_lossy(prefixed(&self.bytes(index)?, 1)?).into_owned())
    }

    fn value(&self, index: &[u8]) -> Result<DbValue, DbError> {
        let kind = index[15] >> 4;
        let size = usize::from(index[15] & 0x0f);
        let record = u64_at(index, 0);

        if size != 0 || record == 0 {
            return decode(kind, &index[..size]);
        }

        let bytes = self.bytes(record)?;
        let elem = match kind {
            STRING_META_VALUE => 1,
            VEC_I64_META_VALUE | VEC_U64_META_VALUE | VEC_F64_META_VALUE => WORD,
            _ => return decode(kind, &bytes),
        };

        decode(kind, prefixed(&bytes, elem)?)
    }

    fn key_value(&self, bytes: &[u8]) -> Result<DbKeyValue, DbError> {
        Ok(DbKeyValue {
            key: self.value(&bytes[..VALUE as usize])?,
            value: self.value(&bytes[VALUE as usize..])?,
        })
    }
}

impl<V: View> Source for Legacy<'_, V> {
    fn slots(&self) -> u64 {
        self.fields.as_ref().map_or(0, |fields| fields.slots)
    }

    fn graph(&self, start: u64, count: u64) -> Result<Vec<Element>, DbError> {
        let mut elements: Vec<Element> = (0..count).map(|_| (None, None)).collect();
        let Some(fields) = &self.fields else {
            return Ok(elements);
        };
        let first = start.min(fields.slots);
        let len = start.saturating_add(count).min(fields.slots) - first;
        let columns = fields
            .columns
            .iter()
            .map(|column| self.elements(*column, first, len, WORD))
            .collect::<Result<Vec<StorageSlice>, DbError>>()?;

        for (slot, element) in elements.iter_mut().take(len as usize).enumerate() {
            let field = |column: usize| u64_at(&columns[column], slot) as i64;
            *element = self::element(field(0), field(1), field(2), field(3));
        }

        Ok(elements)
    }

    fn indexes(&self) -> Result<Vec<DbValue>, DbError> {
        let Some(fields) = &self.fields else {
            return Ok(vec![]);
        };
        let len = self.word(fields.indexes, 0)?;

        self.elements(fields.indexes, 0, len, INDEX)?
            .chunks(INDEX as usize)
            .map(|index| self.value(&index[..VALUE as usize]))
            .collect()
    }

    /// The aliases of the nodes in the slot order of the multimap of
    /// aliases. Those of edges and missing nodes, empty ones, a second
    /// alias of a node and a second node of an alias are `fixed`.
    fn aliases(&self, fixed: &mut Vec<String>) -> Result<Vec<(DbId, String)>, DbError> {
        let Some(fields) = &self.fields else {
            return Ok(vec![]);
        };
        let mut aliases = vec![];
        let mut by_id = HashMap::<DbId, String>::new();
        let mut by_alias = HashMap::<String, DbId>::new();

        for entry in self.map(fields.aliases, 0, u64::MAX, WORD, WORD)? {
            let alias = self.string(u64_at(&entry.0, 0))?;
            let id = DbId(u64_at(&entry.1, 0) as i64);
            let (node, edge) = self.kind(id)?;
            let message = if edge {
                format!(
                    "alias '{alias}' of edge {} removed: edges cannot have aliases",
                    id.0
                )
            } else if !node {
                format!("alias '{alias}' of missing element {} removed", id.0)
            } else if alias.is_empty() {
                format!(
                    "empty alias of node {} removed: aliases cannot be empty",
                    id.0
                )
            } else if let Some(other) = by_id.get(&id) {
                format!(
                    "alias '{alias}' of node {} removed: the node already has alias '{other}'",
                    id.0
                )
            } else if let Some(owner) = by_alias.get(&alias) {
                format!(
                    "alias '{alias}' of node {} removed: node {} already has it",
                    id.0, owner.0
                )
            } else {
                by_id.insert(id, alias.clone());
                by_alias.insert(alias.clone(), id);
                aliases.push((id, alias));
                continue;
            };

            fixed.push(message);
        }

        Ok(aliases)
    }

    /// The elements of the vector of key-value lists by id, the slots of
    /// the multimap of the version 0.
    fn value_slots(&self) -> Result<u64, DbError> {
        let Some(fields) = &self.fields else {
            return Ok(0);
        };

        if self.version == 0 {
            return self.word(self.word(fields.values, WORD)?, 0);
        }

        self.word(fields.values, 0)
    }

    /// The key-values of the value slots `start..start + count` in their
    /// order. Those of missing elements are `fixed`.
    fn values(
        &self,
        start: u64,
        count: u64,
        fixed: &mut Vec<String>,
    ) -> Result<Vec<(DbId, DbKeyValue)>, DbError> {
        let Some(fields) = &self.fields else {
            return Ok(vec![]);
        };
        let mut values = vec![];

        if self.version == 0 {
            for entry in self.map(fields.values, start, count, WORD, KEY_VALUE)? {
                let id = DbId(u64_at(&entry.0, 0) as i64);
                let (node, edge) = self.kind(id)?;
                values.push((id, node || edge, self.key_value(&entry.1)?));
            }
        } else {
            let len = self.word(fields.values, 0)?;
            let start = start.min(len);
            let count = count.min(len - start);
            let lists = self.elements(fields.values, start, count, WORD)?;

            for (at, element) in self.graph(start, count)?.into_iter().enumerate() {
                let list = u64_at(&lists, at);
                let slot = (start + at as u64) as i64;
                let id = DbId(if element.1.is_some() { -slot } else { slot });

                if list != 0 {
                    let len = self.word(list, 0)?;

                    for key_value in self
                        .elements(list, 0, len, KEY_VALUE)?
                        .chunks(KEY_VALUE as usize)
                    {
                        let exists = element.0.is_some() || element.1.is_some();
                        values.push((id, exists, self.key_value(key_value)?));
                    }
                }
            }
        }

        Ok(values
            .into_iter()
            .filter_map(|value| {
                if !value.1 {
                    fixed.push(format!(
                        "value of key '{}' of missing element {} removed",
                        value.2.key, value.0.0
                    ));
                }

                value.1.then_some((value.0, value.2))
            })
            .collect())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::db::db_value::BYTES_META_VALUE;
    use crate::db::db_value::F64_META_VALUE;
    use crate::db::db_value::I64_META_VALUE;
    use crate::db::db_value::NONE_META_VALUE;
    use crate::db::db_value::U64_META_VALUE;
    use crate::db::db_value::VEC_DBVALUE_META_VALUE;
    use crate::db::db_value::VEC_STRING_META_VALUE;
    use crate::storage::alloc::tests::Faulty;

    /// A fault that never fails, so every test reads through the view of the sweep.
    const NEVER: u64 = u64::MAX;
    use crate::storage::memory_storage::MemoryStorageImpl;
    use crate::storage::pages::Reader;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_platform;
    use std::sync::Arc;

    type Storage = MemoryStorageImpl<TestPlatform>;

    pub(crate) const FREE: i64 = i64::MIN;
    /// The slots of the test graph as `[from, to, from_meta, to_meta]`:
    /// nodes 1 and 2, the edge -3 from 1 to 2, the free slot 4 and the
    /// edge -5 from 2 to 2.
    pub(crate) const GRAPH: [[i64; 4]; 6] = [
        [0, 0, -4, 2],
        [3, 0, 1, 0],
        [5, 5, 1, 2],
        [-1, -2, 0, 0],
        [0, 0, FREE, 0],
        [-2, -2, 0, 3],
    ];
    /// The multimap of aliases of the test database as (state, alias, id).
    const ALIASES: [(u8, &str, i64); 13] = [
        (1, "a", 1),
        (0, "", 0),
        (2, "", 1),
        (1, "e", -3),
        (1, "m", 9),
        (1, "f", 4),
        (1, "z", 0),
        (1, "n", -1),
        (1, "p", 3),
        (1, "", 2),
        (1, "b", 1),
        (1, "a", 2),
        (1, "c", 2),
    ];

    pub(crate) fn words(words: &[u64]) -> Vec<u8> {
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }

    pub(crate) fn signed(words: &[i64]) -> Vec<u8> {
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }

    /// The value index of `kind` holding `bytes`.
    pub(crate) fn inline(kind: u8, bytes: &[u8]) -> Vec<u8> {
        let mut index = vec![0; 16];
        index[..bytes.len()].copy_from_slice(bytes);
        index[15] = kind << 4 | bytes.len() as u8;
        index
    }

    /// The value index of `kind` stored in the record `record`.
    pub(crate) fn stored(kind: u8, record: u64) -> Vec<u8> {
        let mut index = words(&[record, 0]);
        index[15] = kind << 4;
        index
    }

    pub(crate) fn string(value: &str) -> Vec<u8> {
        inline(STRING_META_VALUE, value.as_bytes())
    }

    /// Records written in order, from the index 2 as the root is 1.
    pub(crate) struct File {
        pub(crate) bytes: Vec<u8>,
        next: u64,
    }

    impl File {
        /// Of the storage version 1, without the version record for 0.
        pub(crate) fn new(storage: u64) -> Self {
            Self {
                bytes: if storage == 0 {
                    vec![]
                } else {
                    words(&[0, 8, storage])
                },
                next: 2,
            }
        }

        pub(crate) fn put(&mut self, index: u64, value: &[u8]) {
            self.bytes.extend(words(&[index, value.len() as u64]));
            self.bytes.extend(value);
        }

        pub(crate) fn add(&mut self, value: &[u8]) -> u64 {
            let index = self.next;
            self.next += 1;
            self.put(index, value);
            index
        }

        pub(crate) fn vec(&mut self, len: u64, elements: &[u8]) -> u64 {
            self.add(&[words(&[len]), elements.to_vec()].concat())
        }

        pub(crate) fn string(&mut self, value: &str) -> u64 {
            self.vec(value.len() as u64, value.as_bytes())
        }

        pub(crate) fn graph(&mut self, slots: &[[i64; 4]]) -> u64 {
            let columns: Vec<u64> = (0..4)
                .map(|column| {
                    let elements: Vec<i64> = slots.iter().map(|slot| slot[column]).collect();
                    self.vec(slots.len() as u64, &signed(&elements))
                })
                .collect();
            self.add(&words(&columns))
        }

        /// A multimap of the (state, key, value) `entries` with the stale
        /// length 0.
        pub(crate) fn map(&mut self, entries: &[(u8, Vec<u8>, Vec<u8>)]) -> u64 {
            let len = entries.len() as u64;
            let states: Vec<u8> = entries.iter().map(|entry| entry.0).collect();
            let keys: Vec<u8> = entries.iter().flat_map(|entry| entry.1.clone()).collect();
            let values: Vec<u8> = entries.iter().flat_map(|entry| entry.2.clone()).collect();
            let header = [
                0,
                self.vec(len, &states),
                self.vec(len, &keys),
                self.vec(len, &values),
            ];
            self.add(&words(&header))
        }

        pub(crate) fn aliases(&mut self) -> u64 {
            let entries: Vec<(u8, Vec<u8>, Vec<u8>)> = ALIASES
                .iter()
                .map(|alias| {
                    let key = if alias.0 == VALID {
                        self.string(alias.1)
                    } else {
                        0
                    };
                    (alias.0, words(&[key]), signed(&[alias.2]))
                })
                .collect();
            self.map(&entries)
        }

        pub(crate) fn storage(&self) -> Storage {
            MemoryStorageImpl::from_buffer_with_platform(
                Arc::new(test_platform()),
                "/legacy",
                self.bytes.clone(),
            )
        }
    }

    /// The key-values of every value type, inline and stored, with their
    /// legacy bytes.
    pub(crate) fn key_values(file: &mut File) -> (Vec<u8>, Vec<DbKeyValue>) {
        let long = "a key longer than the 15 inline bytes";
        let entries = [
            (string("none"), inline(NONE_META_VALUE, &[]), DbValue::None),
            (
                string("bytes"),
                inline(BYTES_META_VALUE, &[1, 2]),
                DbValue::Bytes(vec![1, 2]),
            ),
            (
                string("blob"),
                stored(BYTES_META_VALUE, file.add(&[7; 16])),
                DbValue::Bytes(vec![7; 16]),
            ),
            (
                string("i64"),
                inline(I64_META_VALUE, &(-1_i64).to_le_bytes()),
                DbValue::I64(-1),
            ),
            (
                string("u64"),
                inline(U64_META_VALUE, &2_u64.to_le_bytes()),
                DbValue::U64(2),
            ),
            (
                string("f64"),
                inline(F64_META_VALUE, &1.5_f64.to_le_bytes()),
                DbValue::from(1.5),
            ),
            (string("short"), string("s"), DbValue::from("s")),
            (
                stored(STRING_META_VALUE, file.string(long)),
                stored(STRING_META_VALUE, file.string(long)),
                DbValue::from(long),
            ),
            (
                string("vec_i64"),
                stored(VEC_I64_META_VALUE, file.vec(2, &signed(&[1, -1]))),
                DbValue::from(vec![1_i64, -1]),
            ),
            (
                string("vec_u64"),
                stored(VEC_U64_META_VALUE, file.vec(1, &words(&[3]))),
                DbValue::from(vec![3_u64]),
            ),
            (
                string("vec_f64"),
                stored(VEC_F64_META_VALUE, file.vec(1, &2.5_f64.to_le_bytes())),
                DbValue::from(vec![2.5]),
            ),
            (
                string("vec_string"),
                stored(
                    VEC_STRING_META_VALUE,
                    file.vec(
                        2,
                        &[
                            &words(&[1])[..],
                            "x".as_bytes(),
                            &words(&[2]),
                            "yy".as_bytes(),
                        ]
                        .concat(),
                    ),
                ),
                DbValue::from(vec!["x", "yy"]),
            ),
            (
                string("vec_value"),
                stored(
                    VEC_DBVALUE_META_VALUE,
                    file.vec(1, &[&[1][..], &(-1_i64).to_le_bytes()].concat()),
                ),
                DbValue::VecDbValue(vec![DbValue::I64(-1)]),
            ),
        ];
        let bytes = entries
            .iter()
            .flat_map(|entry| [entry.0.clone(), entry.1.clone()].concat())
            .collect();
        let key_values = entries
            .into_iter()
            .map(|entry| DbKeyValue {
                key: Legacy::open(&Faulty::new(&mut Reader::new(&file.storage()), NEVER))
                    .unwrap()
                    .value(&entry.0)
                    .unwrap(),
                value: entry.2,
            })
            .collect();
        (bytes, key_values)
    }

    /// The test database of the version 1: GRAPH, ALIASES, the index "k",
    /// the key-values of every type on the node 2 and lingering ones on
    /// the free slot 4 and the slot 6 past the graph, with the expected
    /// key-values.
    pub(crate) fn database() -> (File, Vec<(DbId, DbKeyValue)>) {
        let mut file = File::new(1);
        let graph = file.graph(&GRAPH);
        let aliases = file.aliases();
        let indexes = file.vec(1, &[string("k"), words(&[0])].concat());
        let (bytes, all) = key_values(&mut file);
        let list = |file: &mut File, key: &str, value: u64| {
            let entry = [string(key), inline(U64_META_VALUE, &value.to_le_bytes())].concat();
            file.vec(1, &entry)
        };
        let lists = [
            0,
            list(&mut file, "k", 1),
            file.vec(all.len() as u64, &bytes),
            list(&mut file, "w", 1),
            list(&mut file, "x", 1),
            file.vec(0, &[]),
            list(&mut file, "y", 1),
        ];
        let values = file.vec(lists.len() as u64, &words(&lists));
        file.put(ROOT, &words(&[1, graph, aliases, 0, indexes, values]));
        let mut expected = vec![(DbId(1), ("k", 1_u64).into())];
        expected.extend(all.into_iter().map(|key_value| (DbId(2), key_value)));
        expected.push((DbId(-3), ("w", 1_u64).into()));
        (file, expected)
    }

    /// The test database of the version 0 in the storage `storage`: the
    /// GRAPH and a multimap of key-values with the free slot 4 and a
    /// deleted slot.
    pub(crate) fn database_v0(storage: u64) -> File {
        let mut file = File::new(storage);
        let graph = file.graph(&GRAPH);
        let entry = |key: &str, value: u64| {
            [string(key), inline(U64_META_VALUE, &value.to_le_bytes())].concat()
        };
        let values = file.map(&[
            (1, signed(&[1]), entry("k", 1)),
            (2, signed(&[2]), vec![0; 32]),
            (1, signed(&[-3]), entry("w", 1)),
            (1, signed(&[4]), entry("x", 1)),
            (1, signed(&[2]), entry("v", 2)),
        ]);
        let aliases = file.map(&[]);
        let indexes = file.vec(0, &[]);
        file.put(ROOT, &words(&[graph, aliases, 0, indexes, values]));
        file
    }

    fn fixed_values() -> Vec<String> {
        vec!["value of key 'x' of missing element 4 removed".to_string()]
    }

    /// Reads all of the `view`.
    fn read_all(view: &impl View) -> Result<(), DbError> {
        let legacy = Legacy::open(view)?;
        legacy.graph(0, legacy.slots())?;
        legacy.indexes()?;
        legacy.aliases(&mut vec![])?;
        legacy.values(0, legacy.value_slots()?, &mut vec![])?;
        Ok(())
    }

    mod legacy {
        use super::*;

        #[test]
        fn legacy_failed_reads_should_fail() {
            for storage in [database().0.storage(), database_v0(1).storage()] {
                let mut reader = Reader::new(&storage);
                let mut fail = 0;

                while read_all(&Faulty::new(&mut reader, fail)).is_err() {
                    fail += 1;
                    assert!(fail < 1000, "{:?}", read_all(&reader));
                }

                assert!(30 < fail, "fail: {fail}");
            }
        }

        mod open {
            use super::*;

            #[test]
            fn open_should_read_both_versions() {
                for (file, version) in [(database().0, 1), (database_v0(1), 0), (database_v0(0), 0)]
                {
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(legacy.version(), version);
                    assert_eq!(legacy.slots(), 6);
                }
            }

            #[test]
            fn open_without_root_should_be_empty() {
                for file in [File::new(1), File::new(0)] {
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(legacy.version(), 1);
                    assert_eq!(legacy.slots(), 0);
                    assert_eq!(
                        legacy.graph(1, 2).unwrap(),
                        vec![(None, None), (None, None)]
                    );
                    assert_eq!(legacy.indexes().unwrap(), vec![]);
                    assert_eq!(legacy.aliases(&mut vec![]).unwrap(), vec![]);
                    assert_eq!(legacy.value_slots().unwrap(), 0);
                    assert_eq!(legacy.values(0, 1, &mut vec![]).unwrap(), vec![]);
                }
            }

            #[test]
            fn open_should_skip_free_records_and_take_the_later_of_an_index() {
                let mut file = File::new(1);
                file.put(0, &[9; 3]);
                file.put(2, &words(&[7]));
                file.put(2, &words(&[8]));
                let storage = file.storage();
                let mut reader = Reader::new(&storage);
                let view = Faulty::new(&mut reader, NEVER);
                let legacy = Legacy::open(&view).unwrap();

                assert_eq!(legacy.word(2, 0).unwrap(), 8);
                assert_eq!(legacy.records.len(), 1);
            }

            #[test]
            fn open_invalid_storage_should_fail() {
                let root = |file: &mut File, root: &[u64]| {
                    file.put(ROOT, &words(root));
                };
                let mut cases: Vec<(Vec<u8>, &str)> = vec![
                    (vec![0; 8], "Read (0..16) out of bounds (8)"),
                    (words(&[0, 4]), "Invalid version record size (4 < 8)"),
                    (words(&[0, 8]), "Read (16..24) out of bounds (16)"),
                    (
                        words(&[0, 8, 1, 1, 100]),
                        "Invalid record size (100) exceeds remaining storage (0)",
                    ),
                    (
                        [words(&[0, 8, 1]), vec![1, 2]].concat(),
                        "Read (24..40) out of bounds (26)",
                    ),
                ];
                let mut add = |description: &'static str, build: &dyn Fn(&mut File)| {
                    let mut file = File::new(1);
                    build(&mut file);
                    cases.push((file.bytes, description));
                };
                add(
                    "Database version '2' is not supported by storage version '1'",
                    &|file| root(file, &[2, 0, 0, 0, 0, 0]),
                );
                add("Value size (8) out of bounds (4)", &|file| {
                    file.put(ROOT, &[0; 4])
                });
                add("Value size (48) out of bounds (44)", &|file| {
                    file.put(ROOT, &[&words(&[1, 0, 0, 0, 0])[..], &[0; 4]].concat())
                });
                add("Index (9) not found", &|file| {
                    root(file, &[1, 9, 0, 0, 0, 0])
                });
                add(
                    "Graph v1 vectors have different lengths (1 != 2)",
                    &|file| {
                        let two = file.vec(2, &[0; 16]);
                        let one = file.vec(1, &[0; 8]);
                        let graph = file.add(&words(&[two, two, one, two]));
                        root(file, &[1, graph, 0, 0, 0, 0]);
                    },
                );
                add("Graph v1 vectors are empty", &|file| {
                    let empty = file.vec(0, &[]);
                    let graph = file.add(&words(&[empty; 4]));
                    root(file, &[1, graph, 0, 0, 0, 0]);
                });

                for (bytes, description) in cases {
                    let storage = MemoryStorageImpl::from_buffer_with_platform(
                        Arc::new(test_platform()),
                        "/legacy",
                        bytes,
                    );

                    assert_eq!(
                        Legacy::open(&Faulty::new(&mut Reader::new(&storage), NEVER))
                            .err()
                            .map(|error| error.description),
                        Some(description.to_string())
                    );
                }
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_invalid_should_fail() {
                let mut file = File::new(1);
                let short = file.add(&words(&[5]));
                let storage = file.storage();
                let mut reader = Reader::new(&storage);
                let view = Faulty::new(&mut reader, NEVER);
                let legacy = Legacy::open(&view).unwrap();

                for (index, description) in [
                    (inline(11, &[]), "Invalid value type '11'"),
                    (stored(STRING_META_VALUE, 9), "Index (9) not found"),
                    (
                        stored(STRING_META_VALUE, short),
                        "Invalid vector length (8 bytes)",
                    ),
                ] {
                    assert_eq!(legacy.value(&index).unwrap_err().description, description);
                }
            }
        }

        mod source {
            use super::*;

            mod graph {
                use super::*;

                #[test]
                fn graph_should_return_elements_of_slots() {
                    let file = database().0;
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();
                    let elements = legacy.graph(0, 8).unwrap();

                    assert_eq!(elements.len(), 8);
                    assert_eq!(
                        elements[2],
                        element(GRAPH[2][0], GRAPH[2][1], GRAPH[2][2], GRAPH[2][3])
                    );
                    assert_eq!(
                        elements
                            .iter()
                            .map(|element| (element.0.is_some(), element.1.is_some()))
                            .collect::<Vec<_>>(),
                        vec![
                            (false, false),
                            (true, false),
                            (true, false),
                            (false, true),
                            (false, false),
                            (false, true),
                            (false, false),
                            (false, false),
                        ]
                    );
                    assert_eq!(legacy.graph(4, 1).unwrap(), vec![(None, None)]);
                    assert_eq!(
                        legacy.graph(7, 2).unwrap(),
                        vec![(None, None), (None, None)]
                    );
                }
            }

            mod indexes {
                use super::*;

                #[test]
                fn indexes_should_return_keys() {
                    let storage = database().0.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(legacy.indexes().unwrap(), vec![DbValue::from("k")]);
                }
            }

            mod aliases {
                use super::*;

                #[test]
                fn aliases_should_keep_first_alias_of_nodes_and_fix_others() {
                    let storage = database().0.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();
                    let mut fixed = vec![];

                    assert_eq!(
                        legacy.aliases(&mut fixed).unwrap(),
                        vec![(DbId(1), "a".to_string()), (DbId(2), "c".to_string())]
                    );
                    assert_eq!(
                        fixed,
                        vec![
                            "alias 'e' of edge -3 removed: edges cannot have aliases",
                            "alias 'm' of missing element 9 removed",
                            "alias 'f' of missing element 4 removed",
                            "alias 'z' of missing element 0 removed",
                            "alias 'n' of missing element -1 removed",
                            "alias 'p' of missing element 3 removed",
                            "empty alias of node 2 removed: aliases cannot be empty",
                            "alias 'b' of node 1 removed: the node already has alias 'a'",
                            "alias 'a' of node 2 removed: node 1 already has it",
                        ]
                    );
                }

                #[test]
                fn aliases_invalid_should_fail() {
                    let mut file = File::new(1);
                    let graph = file.graph(&GRAPH);
                    let short = file.add(&words(&[5]));
                    let aliases = file.map(&[(VALID, words(&[short]), signed(&[1]))]);
                    let empty = file.vec(0, &[]);
                    file.put(ROOT, &words(&[1, graph, aliases, 0, empty, empty]));
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(
                        legacy.aliases(&mut vec![]).unwrap_err().description,
                        "Invalid vector length (8 bytes)"
                    );
                }
            }

            mod value_slots {
                use super::*;

                #[test]
                fn value_slots_should_count_both_versions() {
                    for (file, slots) in [(database().0, 7), (database_v0(1), 5)] {
                        let storage = file.storage();
                        let mut reader = Reader::new(&storage);
                        let view = Faulty::new(&mut reader, NEVER);
                        let legacy = Legacy::open(&view).unwrap();

                        assert_eq!(legacy.value_slots().unwrap(), slots);
                    }
                }
            }

            mod values {
                use super::*;

                #[test]
                fn values_should_return_key_values_of_existing_elements() {
                    let (file, expected) = database();
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();
                    let mut fixed = vec![];

                    assert_eq!(legacy.values(0, 9, &mut fixed).unwrap(), expected);
                    assert_eq!(
                        fixed,
                        [
                            fixed_values(),
                            vec!["value of key 'y' of missing element 6 removed".to_string()]
                        ]
                        .concat()
                    );
                }

                #[test]
                fn values_of_slots_should_return_their_key_values() {
                    let (file, expected) = database();
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(
                        legacy.values(2, 2, &mut vec![]).unwrap(),
                        expected[1..].to_vec()
                    );
                    assert_eq!(legacy.values(8, 2, &mut vec![]).unwrap(), vec![]);
                }

                #[test]
                fn values_v0_should_return_key_values_in_slot_order() {
                    let storage = database_v0(1).storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();
                    let mut fixed = vec![];

                    assert_eq!(
                        legacy.values(0, 9, &mut fixed).unwrap(),
                        vec![
                            (DbId(1), ("k", 1_u64).into()),
                            (DbId(-3), ("w", 1_u64).into()),
                            (DbId(2), ("v", 2_u64).into()),
                        ]
                    );
                    assert_eq!(fixed, fixed_values());
                    assert_eq!(
                        legacy.values(3, 9, &mut vec![]).unwrap(),
                        vec![(DbId(2), ("v", 2_u64).into())]
                    );
                }

                #[test]
                fn values_v0_invalid_should_fail() {
                    let mut file = File::new(1);
                    let graph = file.graph(&GRAPH);
                    let entry = [inline(11, &[]), inline(U64_META_VALUE, &[1])].concat();
                    let values = file.map(&[(1, signed(&[1]), entry)]);
                    let aliases = file.map(&[]);
                    let indexes = file.vec(0, &[]);
                    file.put(ROOT, &words(&[graph, aliases, 0, indexes, values]));
                    let storage = file.storage();
                    let mut reader = Reader::new(&storage);
                    let view = Faulty::new(&mut reader, NEVER);
                    let legacy = Legacy::open(&view).unwrap();

                    assert_eq!(
                        legacy.values(0, 9, &mut vec![]).unwrap_err().description,
                        "Invalid value type '11'"
                    );
                }
            }
        }
    }

    mod prefixed {
        use super::*;

        #[test]
        fn prefixed_should_return_elements_after_length() {
            assert_eq!(
                prefixed(&[&words(&[2])[..], &[1, 2, 3]].concat(), 1).unwrap(),
                &[1, 2]
            );
        }

        #[test]
        fn prefixed_invalid_length_should_fail() {
            for (bytes, elem) in [
                (vec![1, 0, 0, 0], 1),
                (words(&[u64::MAX]), 2),
                (words(&[u64::MAX - 3]), 1),
                ([words(&[3]), vec![1, 2]].concat(), 1),
            ] {
                assert_eq!(
                    prefixed(&bytes, elem).unwrap_err().description,
                    format!("Invalid vector length ({} bytes)", bytes.len())
                );
            }
        }
    }

    mod element {
        use super::*;

        #[test]
        fn element_should_tell_free_slots_nodes_and_edges() {
            assert_eq!(element(0, 0, FREE, 0), (None, None));
            assert_eq!(element(0, 0, -2, 0), (None, None));
            assert_eq!(
                element(3, 5, 1, 2),
                (
                    Some(Node {
                        first_from: DbId(-3),
                        first_to: DbId(-5),
                        from_count: 0,
                        to_count: 0,
                        data: 0,
                    }),
                    None
                )
            );
            assert_eq!(
                element(-2, -1, 3, 0),
                (
                    None,
                    Some(Edge {
                        from: DbId(2),
                        to: DbId(1),
                        next_from: DbId(-3),
                        next_to: DbId(0),
                        previous_from: DbId(0),
                        previous_to: DbId(0),
                        data: 0,
                    })
                )
            );
        }
    }
}
