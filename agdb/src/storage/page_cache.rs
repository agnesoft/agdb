use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

pub(crate) const CACHE_PAGE_SIZE: u64 = 4096;

const NULL: u32 = u32::MAX;
const FIBONACCI: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Default)]
struct PageHasher(u64);

impl Hasher for PageHasher {
    fn finish(&self) -> u64 {
        // WHY: the map takes the low bits, zero for pages stepping by a power of two.
        self.0 ^ (self.0 >> 32)
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0.rotate_left(8) ^ u64::from(byte)).wrapping_mul(FIBONACCI);
        }
    }

    fn write_u64(&mut self, page: u64) {
        self.0 = page.wrapping_mul(FIBONACCI);
    }
}

struct Slot {
    page: u64,
    prev: u32,
    next: u32,
}

pub(crate) struct PageCache {
    buffer: Vec<u8>,
    slots: Vec<Slot>,
    map: HashMap<u64, u32, BuildHasherDefault<PageHasher>>,
    free: Vec<u32>,
    head: u32,
    tail: u32,
}

impl PageCache {
    pub(crate) fn new(capacity: u64) -> Option<Self> {
        let slots = (capacity / CACHE_PAGE_SIZE).min(u64::from(NULL - 1)) as u32;

        if slots == 0 {
            return None;
        }

        Some(Self {
            buffer: vec![0; slots as usize * CACHE_PAGE_SIZE as usize],
            slots: (0..slots)
                .map(|_| Slot {
                    page: u64::MAX,
                    prev: NULL,
                    next: NULL,
                })
                .collect(),
            map: HashMap::with_capacity_and_hasher(slots as usize, BuildHasherDefault::default()),
            free: (0..slots).rev().collect(),
            head: NULL,
            tail: NULL,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.free = (0..self.slots.len() as u32).rev().collect();
        self.head = NULL;
        self.tail = NULL;
    }

    pub(crate) fn get(&mut self, page: u64) -> Option<&[u8]> {
        let slot = *self.map.get(&page)?;

        if self.head != slot {
            self.unlink(slot);
            self.push_head(slot);
        }

        Some(self.bytes(slot))
    }

    pub(crate) fn insert(&mut self, page: u64, bytes: &[u8]) {
        let slot = if let Some(&slot) = self.map.get(&page) {
            self.unlink(slot);
            slot
        } else {
            let slot = self.free.pop().unwrap_or_else(|| {
                let tail = self.tail;
                self.unlink(tail);
                self.map.remove(&self.slots[tail as usize].page);
                tail
            });
            self.slots[slot as usize].page = page;
            self.map.insert(page, slot);
            slot
        };

        let start = slot as usize * CACHE_PAGE_SIZE as usize;
        let buffer = &mut self.buffer[start..start + CACHE_PAGE_SIZE as usize];
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()..].fill(0);
        self.push_head(slot);
    }

    pub(crate) fn invalidate(&mut self, pos: u64, end: u64) {
        if end <= pos || self.map.is_empty() {
            return;
        }

        let first = pos / CACHE_PAGE_SIZE;
        let last = (end - 1) / CACHE_PAGE_SIZE;

        if last - first < self.map.len() as u64 {
            for page in first..=last {
                self.remove(page);
            }
        } else {
            let pages: Vec<u64> = self
                .map
                .keys()
                .copied()
                .filter(|page| (first..=last).contains(page))
                .collect();

            for page in pages {
                self.remove(page);
            }
        }
    }

    fn bytes(&self, slot: u32) -> &[u8] {
        let start = slot as usize * CACHE_PAGE_SIZE as usize;
        &self.buffer[start..start + CACHE_PAGE_SIZE as usize]
    }

    fn push_head(&mut self, slot: u32) {
        self.slots[slot as usize].prev = NULL;
        self.slots[slot as usize].next = self.head;

        if self.head == NULL {
            self.tail = slot;
        } else {
            self.slots[self.head as usize].prev = slot;
        }

        self.head = slot;
    }

    fn remove(&mut self, page: u64) {
        if let Some(slot) = self.map.remove(&page) {
            self.unlink(slot);
            self.free.push(slot);
        }
    }

    fn unlink(&mut self, slot: u32) {
        let Slot { prev, next, .. } = self.slots[slot as usize];

        if prev == NULL {
            self.head = next;
        } else {
            self.slots[prev as usize].next = next;
        }

        if next == NULL {
            self.tail = prev;
        } else {
            self.slots[next as usize].prev = prev;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: usize = CACHE_PAGE_SIZE as usize;

    fn page(byte: u8) -> Vec<u8> {
        vec![byte; PAGE]
    }

    fn cached(cache: &mut PageCache, page: u64) -> Option<u8> {
        cache.get(page).map(|bytes| bytes[0])
    }

    mod page_hasher {
        use super::*;

        mod hasher {
            use super::*;

            mod finish {
                use super::*;

                #[test]
                fn finish_should_spread_bytes_and_pages() {
                    let mut bytes = PageHasher::default();
                    let mut other = PageHasher::default();
                    let mut page = PageHasher::default();

                    bytes.write(&[1, 2]);
                    other.write(&[2, 1]);
                    page.write_u64(1);

                    assert_ne!(bytes.finish(), other.finish());
                    assert_eq!(page.finish(), FIBONACCI ^ (FIBONACCI >> 32));
                }

                #[test]
                fn finish_should_spread_pages_stepping_by_power_of_two_over_low_bits() {
                    let buckets = (0..256_u64)
                        .map(|page| {
                            let mut hasher = PageHasher::default();
                            hasher.write_u64(page << 20);
                            hasher.finish() & 0xff
                        })
                        .collect::<std::collections::HashSet<_>>();

                    assert!(128 < buckets.len(), "{}", buckets.len());
                }
            }
        }
    }

    mod page_cache {
        use super::*;

        #[test]
        fn page_cache_should_evict_least_recently_used_page_when_full() {
            let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE).unwrap();
            cache.insert(0, &page(1));
            cache.insert(1, &page(2));
            cache.get(0);

            cache.insert(2, &page(3));

            assert_eq!(cached(&mut cache, 0), Some(1));
            assert_eq!(cached(&mut cache, 1), None);
            assert_eq!(cached(&mut cache, 2), Some(3));
        }

        #[test]
        fn page_cache_should_reuse_invalidated_slots_before_evicting() {
            let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE).unwrap();
            cache.insert(0, &page(1));
            cache.insert(1, &page(2));
            cache.invalidate(0, 1);

            cache.insert(2, &page(3));

            assert_eq!(cached(&mut cache, 1), Some(2));
            assert_eq!(cached(&mut cache, 2), Some(3));
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_round_capacity_down_to_pages() {
                let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE + 100).unwrap();

                for p in 0..3 {
                    cache.insert(p, &page(p as u8 + 1));
                }

                assert_eq!(cache.buffer.len() as u64, 2 * CACHE_PAGE_SIZE);
                assert_eq!(cached(&mut cache, 0), None);
            }

            #[test]
            fn new_below_page_should_return_none() {
                assert!(PageCache::new(CACHE_PAGE_SIZE - 1).is_none());
            }
        }

        mod clear {
            use super::*;

            #[test]
            fn clear_should_remove_all_pages_keeping_capacity() {
                let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE).unwrap();
                cache.insert(0, &page(1));
                cache.insert(1, &page(2));

                cache.clear();
                cache.insert(2, &page(3));
                cache.insert(3, &page(4));

                assert_eq!(cached(&mut cache, 0), None);
                assert_eq!(cached(&mut cache, 1), None);
                assert_eq!(cached(&mut cache, 2), Some(3));
                assert_eq!(cached(&mut cache, 3), Some(4));
            }
        }

        mod get {
            use super::*;

            #[test]
            fn get_should_return_bytes_of_page() {
                let mut cache = PageCache::new(CACHE_PAGE_SIZE).unwrap();
                cache.insert(7, &page(42));

                assert_eq!(cache.get(7), Some(&page(42)[..]));
            }

            #[test]
            fn get_missing_should_return_none() {
                let mut cache = PageCache::new(CACHE_PAGE_SIZE).unwrap();

                assert_eq!(cache.get(7), None);
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_cached_should_replace_bytes() {
                let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE).unwrap();
                cache.insert(0, &page(1));
                cache.insert(1, &page(2));

                cache.insert(0, &page(3));
                cache.insert(2, &page(4));

                assert_eq!(cached(&mut cache, 0), Some(3));
                assert_eq!(cached(&mut cache, 1), None);
            }

            #[test]
            fn insert_partial_should_zero_rest_of_page() {
                let mut cache = PageCache::new(CACHE_PAGE_SIZE).unwrap();
                cache.insert(0, &page(1));
                cache.invalidate(0, 1);

                cache.insert(0, &[2, 3]);

                let bytes = cache.get(0).unwrap();
                assert_eq!(&bytes[..3], [2, 3, 0]);
                assert!(bytes.iter().skip(2).all(|&byte| byte == 0));
            }
        }

        mod invalidate {
            use super::*;

            #[test]
            fn invalidate_should_remove_pages_overlapping_range() {
                let mut cache = PageCache::new(4 * CACHE_PAGE_SIZE).unwrap();

                for p in 0..4 {
                    cache.insert(p, &page(p as u8 + 1));
                }

                cache.invalidate(CACHE_PAGE_SIZE + 1, 2 * CACHE_PAGE_SIZE + 1);

                assert_eq!(cached(&mut cache, 0), Some(1));
                assert_eq!(cached(&mut cache, 1), None);
                assert_eq!(cached(&mut cache, 2), None);
                assert_eq!(cached(&mut cache, 3), Some(4));
            }

            #[test]
            fn invalidate_range_larger_than_cache_should_scan_cached_pages() {
                let mut cache = PageCache::new(2 * CACHE_PAGE_SIZE).unwrap();
                cache.insert(1, &page(1));
                cache.insert(1000, &page(2));

                cache.invalidate(CACHE_PAGE_SIZE * 500, u64::MAX);

                assert_eq!(cached(&mut cache, 1), Some(1));
                assert_eq!(cached(&mut cache, 1000), None);
            }

            #[test]
            fn invalidate_empty_range_should_do_nothing() {
                let mut cache = PageCache::new(CACHE_PAGE_SIZE).unwrap();
                cache.insert(0, &page(1));

                cache.invalidate(10, 10);

                assert_eq!(cached(&mut cache, 0), Some(1));
            }
        }
    }
}
