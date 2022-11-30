use super::map::map_data_memory::MapDataMemory;
use super::map::multi_map_impl::MultiMapImpl;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;

pub type MultiMap<K, T> = MultiMapImpl<K, T, MapDataMemory<K, T>>;

#[allow(dead_code)]
impl<K, T> MultiMap<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash,
    T: Clone + Default + Eq + PartialEq,
{
    pub fn new() -> Self {
        Self {
            data: MapDataMemory::<K, T>::new(),
            phantom_marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocate() {
        let mut map = MultiMap::<i64, i64>::new();

        assert_eq!(map.capacity(), 0);

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocate_with_collisions() {
        let mut map = MultiMap::<i64, i64>::new();

        for i in 0..50 {
            map.insert(&(i * 64), &i).unwrap();
            map.insert(&(i * 64), &(i + 1)).unwrap();
        }

        for i in 0..50 {
            assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        assert_eq!(map.len(), 2);
        map.insert(&5, &20).unwrap();
        assert_eq!(map.len(), 3);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
    }

    #[test]
    fn iter() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();
        map.insert(&2, &30).unwrap();
        map.insert(&2, &50).unwrap();
        map.insert(&4, &13).unwrap();
        map.remove_key(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
        actual.sort();
        let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (2, 50), (4, 13), (5, 15), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_deleted_key() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);

        map.remove_key(&5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove_key(&5).unwrap();

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn remove_key() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &7).unwrap();
        map.insert(&5, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&5, &20).unwrap();

        assert_eq!(map.len(), 4);
        map.remove_key(&5).unwrap();

        assert_eq!(map.len(), 1);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(Vec::<i64>::new()));
    }

    #[test]
    fn remove_missing_key() {
        let mut map = MultiMap::<i64, i64>::new();

        assert_eq!(map.len(), 0);

        map.remove_key(&5).unwrap();

        assert_eq!(map.len(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let mut map = MultiMap::<i64, i64>::new();

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            map.remove_key(&i).unwrap();
        }

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn remove_value() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&1, &7).unwrap();
        map.insert(&5, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&5, &20).unwrap();

        assert_eq!(map.len(), 4);
        map.remove_value(&5, &15).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(vec![10_i64, 20_i64]));
    }

    #[test]
    fn remove_missing_value() {
        let mut map = MultiMap::<i64, i64>::new();

        map.remove_value(&5, &10).unwrap();

        assert_eq!(map.len(), 0);
    }

    #[test]
    fn replace() {
        let mut map = MultiMap::<i64, i64>::new();
        map.insert(&1, &10).unwrap();
        map.insert(&1, &20).unwrap();
        map.insert(&1, &30).unwrap();

        map.replace(&1, &20, &50).unwrap();

        assert_eq!(map.values(&1), Ok(vec![10, 50, 30]));

        map.remove_value(&1, &50).unwrap();
        map.replace(&1, &30, &40).unwrap();

        assert_eq!(map.values(&1), Ok(vec![10, 40]));
    }

    #[test]
    fn reserve_larger() {
        let mut map = MultiMap::<i64, i64>::new();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let mut map = MultiMap::<i64, i64>::new();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity();
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn reserve_smaller() {
        let mut map = MultiMap::<i64, i64>::new();
        map.insert(&1, &1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn value_missing() {
        let map = MultiMap::<i64, i64>::new();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let mut map = MultiMap::<i64, i64>::new();

        map.insert(&127, &10).unwrap();
        map.insert(&255, &11).unwrap();
        map.insert(&191, &12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
