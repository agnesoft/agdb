use crate::DbError;
use crate::DbErrorType;
use crate::DbValue;
use crate::db::db_f64::DbF64;
use crate::db::db_value::BYTES_META_VALUE;
use crate::db::db_value::F64_META_VALUE;
use crate::db::db_value::I64_META_VALUE;
use crate::db::db_value::NONE_META_VALUE;
use crate::db::db_value::STRING_META_VALUE;
use crate::db::db_value::U64_META_VALUE;
use crate::db::db_value::VEC_DBVALUE_META_VALUE;
use crate::db::db_value::VEC_F64_META_VALUE;
use crate::db::db_value::VEC_I64_META_VALUE;
use crate::db::db_value::VEC_STRING_META_VALUE;
use crate::db::db_value::VEC_U64_META_VALUE;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::u64_at;
use std::borrow::Cow;

// A value is its type and bytes. Up to 15 bytes are inline in its
// index, longer ones are an allocation whose position the index holds
// in bytes 0–7 and its length in bytes 8–14. Numeric vectors are their
// elements as 8 LE bytes each.

pub(crate) fn encode(value: &DbValue) -> (u8, Cow<'_, [u8]>) {
    match value {
        DbValue::Bytes(v) => (BYTES_META_VALUE, Cow::Borrowed(v)),
        DbValue::I64(v) => (I64_META_VALUE, Cow::Owned(v.to_le_bytes().to_vec())),
        DbValue::U64(v) => (U64_META_VALUE, Cow::Owned(v.to_le_bytes().to_vec())),
        DbValue::F64(v) => (
            F64_META_VALUE,
            Cow::Owned(v.to_f64().to_le_bytes().to_vec()),
        ),
        DbValue::String(v) => (STRING_META_VALUE, Cow::Borrowed(v.as_bytes())),
        DbValue::VecI64(v) => (
            VEC_I64_META_VALUE,
            Cow::Owned(v.iter().flat_map(|x| x.to_le_bytes()).collect()),
        ),
        DbValue::VecU64(v) => (
            VEC_U64_META_VALUE,
            Cow::Owned(v.iter().flat_map(|x| x.to_le_bytes()).collect()),
        ),
        DbValue::VecF64(v) => (
            VEC_F64_META_VALUE,
            Cow::Owned(v.iter().flat_map(|x| x.to_f64().to_le_bytes()).collect()),
        ),
        DbValue::VecString(v) => (VEC_STRING_META_VALUE, Cow::Owned(v.serialize())),
        DbValue::VecDbValue(v) => (VEC_DBVALUE_META_VALUE, Cow::Owned(v.serialize())),
        DbValue::None => (NONE_META_VALUE, Cow::Borrowed(&[])),
    }
}

fn words(bytes: &[u8]) -> Result<impl Iterator<Item = u64> + '_, DbError> {
    if !bytes.len().is_multiple_of(8) {
        return Err(DbError::db(
            DbErrorType::InvalidIndex,
            format!("Invalid numeric vector length '{}'", bytes.len()),
        ));
    }

    Ok((0..bytes.len() / 8).map(|word| u64_at(bytes, word)))
}

fn decode(kind: u8, bytes: &[u8]) -> Result<DbValue, DbError> {
    Ok(match kind {
        BYTES_META_VALUE => DbValue::Bytes(bytes.to_vec()),
        I64_META_VALUE => DbValue::I64(i64::deserialize(bytes)?),
        U64_META_VALUE => DbValue::U64(u64::deserialize(bytes)?),
        F64_META_VALUE => DbValue::F64(DbF64::from(f64::deserialize(bytes)?)),
        STRING_META_VALUE => DbValue::String(String::from_utf8_lossy(bytes).into_owned()),
        VEC_I64_META_VALUE => DbValue::VecI64(words(bytes)?.map(|w| w as i64).collect()),
        VEC_U64_META_VALUE => DbValue::VecU64(words(bytes)?.collect()),
        VEC_F64_META_VALUE => {
            DbValue::VecF64(words(bytes)?.map(|w| f64::from_bits(w).into()).collect())
        }
        VEC_STRING_META_VALUE => DbValue::VecString(Vec::deserialize(bytes)?),
        VEC_DBVALUE_META_VALUE => DbValue::VecDbValue(Vec::deserialize(bytes)?),
        NONE_META_VALUE => DbValue::None,
        _ => {
            return Err(DbError::db(
                DbErrorType::InvalidIndex,
                format!("Invalid value type '{kind}'"),
            ));
        }
    })
}

fn stored_len(index: &DbValueIndex) -> u64 {
    let mut len = [0_u8; 8];
    len[..7].copy_from_slice(&index.data()[8..15]);
    u64::from_le_bytes(len)
}

/// Stores the `bytes` of a value of type `kind`.
pub(crate) fn store_bytes(
    view: &mut impl ViewMut,
    alloc: &mut Allocator,
    kind: u8,
    bytes: &[u8],
) -> Result<DbValueIndex, DbError> {
    let mut index = DbValueIndex::new();
    index.set_type(kind);

    if !index.set_value(bytes) {
        let pos = alloc.alloc(view, bytes.len() as u64)?;
        view.write(pos, bytes)?;
        index.set_index(pos);
        let mut data = index.data();
        data[8..15].copy_from_slice(&(bytes.len() as u64).to_le_bytes()[..7]);
        index = DbValueIndex::from_data(data);
    }

    Ok(index)
}

pub(crate) fn store(
    view: &mut impl ViewMut,
    alloc: &mut Allocator,
    value: &DbValue,
) -> Result<DbValueIndex, DbError> {
    let (kind, bytes) = encode(value);
    store_bytes(view, alloc, kind, &bytes)
}

/// The bytes of the stored value.
pub(crate) fn load_bytes(view: &impl View, index: &DbValueIndex) -> Result<Vec<u8>, DbError> {
    if index.is_value() {
        return Ok(index.value().to_vec());
    }

    Ok(view.read(index.index(), stored_len(index))?.to_vec())
}

pub(crate) fn load(view: &impl View, index: &DbValueIndex) -> Result<DbValue, DbError> {
    decode(index.get_type(), &load_bytes(view, index)?)
}

/// Compares the stored value with the `bytes` of a value of type `kind`,
/// reading it only if the types and lengths match.
pub(crate) fn equals_bytes(
    view: &impl View,
    index: &DbValueIndex,
    kind: u8,
    bytes: &[u8],
) -> Result<bool, DbError> {
    if index.get_type() != kind {
        return Ok(false);
    }

    if index.is_value() {
        return Ok(index.value() == bytes);
    }

    Ok(stored_len(index) == bytes.len() as u64
        && *view.read(index.index(), stored_len(index))? == *bytes)
}

pub(crate) fn free(
    view: &mut impl ViewMut,
    alloc: &mut Allocator,
    index: &DbValueIndex,
) -> Result<(), DbError> {
    if index.is_value() {
        return Ok(());
    }

    alloc.free(view, index.index(), stored_len(index))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::storage::alloc::tests::Faulty;
    use crate::storage::alloc::tests::Heap;
    use crate::storage::alloc::tests::heap;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::Overlay;
    use crate::storage::pages::Reader;
    use crate::storage::pages::Writer;

    pub(crate) type TestView<'a, 'b> = Faulty<'a, Writer<'b, Heap>>;

    /// Runs the `test` over a never failing view of an empty heap.
    pub(crate) fn with_view<T>(test: impl FnOnce(&mut TestView, &mut Allocator) -> T) -> T {
        let mut storage = heap();
        let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
        let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
        let mut view = Faulty::new(&mut writer, u64::MAX);
        test(&mut view, &mut Allocator::new(DEFAULT_PAGE_SHIFT))
    }

    fn bytes(view: &impl View) -> Vec<u8> {
        view.read(0, view.len()).unwrap().to_vec()
    }

    /// Fails each read and write of the `action` in turn, on a fresh heap
    /// prepared and committed by the `setup`, until the `action` runs out
    /// of calls to fail. A failed `action` is dropped with its
    /// transaction, which must leave the committed bytes.
    pub(crate) fn assert_failures<T>(
        setup: impl Fn(&mut TestView, &mut Allocator) -> T,
        action: impl Fn(&mut TestView, &mut Allocator, T) -> Result<(), DbError>,
    ) {
        for fail in 0.. {
            let mut storage = heap();
            let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
            let mut alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
            let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
            let state = setup(&mut Faulty::new(&mut writer, u64::MAX), &mut alloc);
            writer.commit().unwrap();
            let committed = bytes(&Reader::new(&storage));
            let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
            let mut view = Faulty::new(&mut writer, u64::MAX);
            view.fail = fail;
            let result = action(&mut view, &mut alloc, state);

            if view.calls.get() <= view.fail {
                result.unwrap();
                break;
            }

            assert!(result.is_err(), "fail: {fail}");
            drop(writer);
            let writer = Writer::new(&mut storage, &mut overlay).unwrap();
            assert_eq!(bytes(&writer), committed, "fail: {fail}");
        }
    }

    /// The (pos, size) of the allocation of a stored value.
    pub(crate) fn allocation(index: &DbValueIndex) -> Option<(u64, u64)> {
        (!index.is_value()).then(|| (index.index(), stored_len(index)))
    }

    /// One value of each type, inline and stored.
    pub(crate) fn values() -> Vec<DbValue> {
        let long = "a long string of more than 15 bytes";
        vec![
            DbValue::Bytes(vec![1, 2, 3]),
            DbValue::Bytes(vec![7; 100]),
            DbValue::I64(-1),
            DbValue::U64(u64::MAX),
            DbValue::F64(1.5.into()),
            DbValue::String(String::new()),
            DbValue::String("alias".to_string()),
            DbValue::String(long.to_string()),
            DbValue::VecI64(vec![]),
            DbValue::VecI64(vec![1, -2]),
            DbValue::VecI64(vec![i64::MIN]),
            DbValue::VecU64(vec![1, 2]),
            DbValue::VecU64(vec![u64::MAX]),
            DbValue::VecF64(vec![1.1.into()]),
            DbValue::VecF64(vec![(-0.5).into(), f64::MAX.into()]),
            DbValue::VecString(vec![long.to_string()]),
            DbValue::VecDbValue(vec![DbValue::I64(1), long.into()]),
            DbValue::None,
        ]
    }

    mod encode {
        use super::*;

        #[test]
        fn encode_numeric_vectors_should_be_le_words() {
            let cases = [
                (
                    DbValue::VecI64(vec![1, -2]),
                    VEC_I64_META_VALUE,
                    [1_i64.to_le_bytes(), (-2_i64).to_le_bytes()].concat(),
                ),
                (DbValue::VecU64(vec![]), VEC_U64_META_VALUE, vec![]),
                (
                    DbValue::VecF64(vec![1.5.into()]),
                    VEC_F64_META_VALUE,
                    1.5_f64.to_le_bytes().to_vec(),
                ),
            ];

            for (value, kind, bytes) in &cases {
                assert_eq!(
                    encode(value),
                    (*kind, Cow::Borrowed(&bytes[..])),
                    "{value:?}"
                );
            }
        }
    }

    mod store {
        use super::*;

        #[test]
        fn store_should_keep_short_values_inline_and_long_ones_allocated() {
            with_view(|view, alloc| {
                for value in values() {
                    let index = store(view, alloc, &value).unwrap();
                    let (_, bytes) = encode(&value);
                    assert_eq!(index.is_value(), bytes.len() <= 15, "{value:?}");
                    assert_eq!(load(view, &index).unwrap(), value);
                }
            });
        }

        #[test]
        fn store_failure_should_fail() {
            assert_failures(
                |_, _| DbValue::Bytes(vec![1; 20]),
                |view, alloc, value| store(view, alloc, &value).map(|_| ()),
            );
        }
    }

    mod load {
        use super::*;

        #[test]
        fn load_corrupt_value_should_fail() {
            with_view(|view, _| {
                for kind in [2, 3, 4, 6, 7, 8, 9, 10, 11] {
                    let mut index = DbValueIndex::new();
                    index.set_type(kind);
                    index.set_value(&[1, 2, 3]);
                    assert!(load(view, &index).is_err(), "{kind}");
                }
            });
        }

        #[test]
        fn load_failure_should_fail() {
            assert_failures(
                |view, alloc| store(view, alloc, &DbValue::Bytes(vec![1; 20])).unwrap(),
                |view, _, index| load(view, &index).map(|_| ()),
            );
        }
    }

    mod equals_bytes {
        use super::*;

        fn equals(view: &TestView, index: &DbValueIndex, value: &DbValue) -> Result<bool, DbError> {
            let (kind, bytes) = encode(value);
            equals_bytes(view, index, kind, &bytes)
        }

        #[test]
        fn equals_bytes_should_compare_type_length_and_bytes() {
            with_view(|view, alloc| {
                let values = values();

                for (i, value) in values.iter().enumerate() {
                    let index = store(view, alloc, value).unwrap();

                    for (j, other) in values.iter().enumerate() {
                        assert_eq!(equals(view, &index, other).unwrap(), i == j, "{i} {j}");
                    }
                }

                let index = store(view, alloc, &DbValue::Bytes(vec![7; 100])).unwrap();
                assert!(!equals(view, &index, &DbValue::Bytes(vec![8; 100])).unwrap());
            });
        }

        #[test]
        fn equals_bytes_failure_should_fail() {
            let value = DbValue::Bytes(vec![1; 20]);
            assert_failures(
                |view, alloc| store(view, alloc, &value).unwrap(),
                |view, _, index| equals(view, &index, &value).map(|_| ()),
            );
        }
    }

    mod free {
        use super::*;

        #[test]
        fn free_should_release_allocation() {
            with_view(|view, alloc| {
                let indexes = values()
                    .iter()
                    .map(|value| store(view, alloc, value).unwrap())
                    .collect::<Vec<_>>();

                for index in &indexes {
                    free(view, alloc, index).unwrap();
                }

                verify(alloc, view, &[]);
            });
        }
    }
}
