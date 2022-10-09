use super::hash_map_data_memory::HashMapDataMemory;
use super::hash_map_impl::HashMapImpl;
use super::hash_map_key_value::HashMapKeyValue;
use super::hash_multi_map_impl::HashMultiMapImpl;
use super::StableHash;
use agdb_serialize::Serialize;
use std::hash::Hash;
use std::marker::PhantomData;

pub(crate) type HashMultiMap<K, T> = HashMultiMapImpl<K, T, HashMapDataMemory<K, T>>;

#[allow(dead_code)]
impl<K, T> HashMultiMap<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    pub(crate) fn new() -> HashMultiMap<K, T> {
        HashMultiMap::<K, T> {
            map: HashMapImpl::<K, T, HashMapDataMemory<K, T>> {
                data: HashMapDataMemory::<K, T> {
                    data: vec![HashMapKeyValue::<K, T>::default()],
                    count: 0,
                },
                phantom_data: PhantomData,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocate() {
        let mut map = HashMultiMap::<i64, i64>::new();

        assert_eq!(map.capacity(), 1);

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.count(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocate_with_collisions() {
        let mut map = HashMultiMap::<i64, i64>::new();

        for i in 0..50 {
            map.insert(i * 64, i).unwrap();
            map.insert(i * 64, i + 1).unwrap();
        }

        for i in 0..50 {
            assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        assert_eq!(map.count(), 2);
        map.insert(5, 20).unwrap();
        assert_eq!(map.count(), 3);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
    }

    #[test]
    fn iter() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();
        map.insert(2, 30).unwrap();
        map.insert(2, 50).unwrap();
        map.insert(4, 13).unwrap();
        map.remove_key(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
        actual.sort();
        let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (2, 50), (4, 13), (5, 15), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_deleted_key() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 2);
    }

    #[test]
    fn remove_key() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 7).unwrap();
        map.insert(5, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 20).unwrap();

        assert_eq!(map.count(), 4);
        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 1);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(Vec::<i64>::new()));
    }

    #[test]
    fn remove_missing_key() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 0);
    }

    #[test]
    fn remove_value() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(1, 7).unwrap();
        map.insert(5, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 20).unwrap();

        assert_eq!(map.count(), 4);
        map.remove_value(&5, &15).unwrap();

        assert_eq!(map.count(), 3);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(vec![10_i64, 20_i64]));
    }

    #[test]
    fn remove_missing_value() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.remove_value(&5, &10).unwrap();

        assert_eq!(map.count(), 0);
    }

    #[test]
    fn remove_missing() {
        let mut map = HashMultiMap::<i64, i64>::new();

        assert_eq!(map.count(), 0);
        assert_eq!(map.remove_key(&0), Ok(()));
        assert_eq!(map.count(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let mut map = HashMultiMap::<i64, i64>::new();

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.count(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            map.remove_key(&i).unwrap();
        }

        assert_eq!(map.count(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let mut map = HashMultiMap::<i64, i64>::new();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.count(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let mut map = HashMultiMap::<i64, i64>::new();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity();
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.count(), size);
    }

    #[test]
    fn reserve_smaller() {
        let mut map = HashMultiMap::<i64, i64>::new();
        map.insert(1, 1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.count(), size);
    }

    #[test]
    fn value_missing() {
        let map = HashMultiMap::<i64, i64>::new();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let mut map = HashMultiMap::<i64, i64>::new();

        map.insert(127, 10).unwrap();
        map.insert(255, 11).unwrap();
        map.insert(191, 12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
