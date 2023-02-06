use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::hash::Hash;

struct IndexedMap<K, T>
where
    K: PartialEq + Eq + Hash + Clone,
    T: PartialEq + Eq + Hash + Clone,
{
    keys_to_values: HashMap<K, T>,
    values_to_keys: HashMap<T, K>,
}

#[allow(dead_code)]
impl<K, T> IndexedMap<K, T>
where
    K: PartialEq + Eq + Hash + Clone,
    T: PartialEq + Eq + Hash + Clone,
{
    pub fn insert(&mut self, key: &K, value: &T) {
        self.keys_to_values.insert((*key).clone(), (*value).clone());
        self.values_to_keys.insert((*value).clone(), (*key).clone());
    }

    pub fn iter(&self) -> Iter<K, T> {
        self.keys_to_values.iter()
    }

    pub fn key(&self, value: &T) -> Option<&K> {
        self.values_to_keys.get(value)
    }

    pub fn new() -> Self {
        Self {
            keys_to_values: HashMap::<K, T>::new(),
            values_to_keys: HashMap::<T, K>::new(),
        }
    }

    pub fn remove_key(&mut self, key: &K) {
        if let Some(value) = self.keys_to_values.get(key) {
            self.values_to_keys.remove(value);
        }

        self.keys_to_values.remove(key);
    }

    pub fn remove_value(&mut self, value: &T) {
        if let Some(key) = self.values_to_keys.get(value) {
            self.keys_to_values.remove(key);
        }

        self.values_to_keys.remove(value);
    }

    pub fn value(&self, key: &K) -> Option<&T> {
        self.keys_to_values.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = IndexedMap::<u64, u64>::new();
    }

    #[test]
    fn insert() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;
        map.insert(&key, &value);

        assert_eq!(map.value(&key), Some(&value));
        assert_eq!(map.key(&value), Some(&key));
    }

    #[test]
    fn iter() {
        let mut map = IndexedMap::<String, u64>::new();
        map.insert(&"alias1".to_string(), &1_u64);
        map.insert(&"alias2".to_string(), &2_u64);
        map.insert(&"alias3".to_string(), &3_u64);

        let mut values = Vec::<(&String, &u64)>::new();

        for key_value in map.iter() {
            values.push(key_value);
        }

        values.sort();

        assert_eq!(
            values,
            vec![
                (&"alias1".to_string(), &1_u64),
                (&"alias2".to_string(), &2_u64),
                (&"alias3".to_string(), &3_u64)
            ]
        );
    }

    #[test]
    fn remove_key() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;
        map.insert(&key, &value);

        assert_eq!(map.value(&key), Some(&value));
        assert_eq!(map.key(&value), Some(&key));

        map.remove_key(&key);

        assert_eq!(map.value(&key), None);
        assert_eq!(map.key(&value), None);
    }

    #[test]
    fn remove_value() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;
        map.insert(&key, &value);

        assert_eq!(map.value(&key), Some(&value));
        assert_eq!(map.key(&value), Some(&key));

        map.remove_value(&value);

        assert_eq!(map.value(&key), None);
        assert_eq!(map.key(&value), None);
    }
}
