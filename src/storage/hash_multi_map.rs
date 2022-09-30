use super::hash_map_data_memory::HashMapDataMemory;
use super::hash_multi_map_impl::HashMultiMapImpl;

type HashMultiMap<K, T> = HashMultiMapImpl<K, T, HashMapDataMemory<K, T>>;
