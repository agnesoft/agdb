use crate::DbError;
use crate::DbErrorType;
use crate::collections::graph::GraphHeader;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

// WHY: (slot, version of the commit that freed it), 0 if free since the pool was made or new
pub(crate) type Reserved = (u64, u64);

#[derive(Debug, Default)]
pub(crate) struct IdPool {
    // WHY: not stored, a taken slot stays in the free list of the storage until
    // its transaction commits, so a crash leaks none.
    slots: Mutex<Option<[Slots; 2]>>,
    made: AtomicBool,
    // WHY: transactions of an older generation may hold slots of the old graph, so
    // their reservations fail and their slots are not taken back.
    generation: AtomicU64,
}

#[derive(Debug)]
struct Slots {
    free: Vec<Reserved>,
    // WHY: the free list past the cursor is not changed by commits, they take only
    // slots handed out before it and free in front of it.
    cursor: u64,
    next: u64,
}

impl IdPool {
    pub(crate) fn init(&self, graph: &GraphHeader) {
        if self.made.load(Ordering::Acquire) {
            return;
        }

        self.slots().get_or_insert_with(|| {
            [false, true].map(|edges| Slots {
                free: Vec::new(),
                cursor: graph.free_head(edges),
                next: graph.len(edges),
            })
        });
        self.made.store(true, Ordering::Release);
    }

    pub(crate) fn made(&self) -> bool {
        self.made.load(Ordering::Acquire)
    }

    pub(crate) fn reset(&self) {
        let mut slots = self.slots();
        *slots = None;
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.made.store(false, Ordering::Release);
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub(crate) fn reserve(
        &self,
        edges: bool,
        (version, generation): (u64, u64),
        count: usize,
        mut next_free: impl FnMut(u64) -> Result<u64, DbError>,
        reserved: &mut Vec<Reserved>,
    ) -> Result<(), DbError> {
        let mut slots = self.slots();

        let Some(slots) = slots.as_mut().filter(|_| self.generation() == generation) else {
            return Err(changed());
        };

        let slots = &mut slots[usize::from(edges)];
        let start = reserved.len();
        let mut result = Ok(());

        for _ in 0..count {
            // WHY: a slot freed after the snapshot of the transaction must not be handed to it
            if let Some(&(slot, freed)) = slots.free.last()
                && freed <= version
            {
                slots.free.pop();
                reserved.push((slot, freed));
            } else if slots.cursor != 0 {
                match next_free(slots.cursor) {
                    Ok(next) => reserved.push((std::mem::replace(&mut slots.cursor, next), 0)),
                    Err(error) => {
                        result = Err(error);
                        break;
                    }
                }
            } else {
                slots.next += 1;
                reserved.push((slots.next - 1, 0));
            }
        }

        reserved[start..].reverse();
        result
    }

    pub(crate) fn finish(
        &self,
        generation: u64,
        unused: [&[Reserved]; 2],
        freed: Option<([&[u64]; 2], u64)>,
    ) {
        if unused.iter().all(|unused| unused.is_empty())
            && freed.is_none_or(|(freed, _)| freed.iter().all(|freed| freed.is_empty()))
        {
            return;
        }

        if let Some(slots) = self
            .slots()
            .as_mut()
            .filter(|_| self.generation() == generation)
        {
            for (kind, slots) in slots.iter_mut().enumerate() {
                slots.free.extend_from_slice(unused[kind]);

                if let Some((freed, version)) = freed {
                    slots
                        .free
                        .extend(freed[kind].iter().map(|slot| (*slot, version)));
                }
            }
        }
    }

    fn slots(&self) -> MutexGuard<'_, Option<[Slots; 2]>> {
        self.slots.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub(crate) fn changed() -> DbError {
    DbError::db(
        DbErrorType::Conflict,
        "The database changed since the transaction started",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbErrorType;
    use crate::DbId;
    use crate::collections::graph::tests::discard;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::with_view;
    use crate::storage::root::Layout;

    fn with_pool(test: impl FnOnce(&IdPool, &GraphHeader, &TestView)) {
        with_view(|view, alloc| {
            let mut graph = GraphHeader::new(Layout::default());

            for _ in 0..5 {
                graph.insert_node(view, alloc).unwrap();
            }

            graph.insert_edge(view, alloc, DbId(1), DbId(1)).unwrap();

            for node in [2, 4] {
                graph.remove_node(view, DbId(node), discard).unwrap();
            }

            let pool = IdPool::default();
            pool.init(&graph);
            test(&pool, &graph, view);
        });
    }

    fn reserve_many(
        pool: &IdPool,
        graph: &GraphHeader,
        view: &TestView,
        edges: bool,
        version: u64,
        count: usize,
    ) -> Vec<Reserved> {
        let mut reserved = Vec::new();
        pool.reserve(
            edges,
            (version, pool.generation()),
            count,
            |slot| graph.next_free(view, edges, slot),
            &mut reserved,
        )
        .unwrap();
        reserved
    }

    fn reserve(
        pool: &IdPool,
        graph: &GraphHeader,
        view: &TestView,
        edges: bool,
        version: u64,
    ) -> Reserved {
        reserve_many(pool, graph, view, edges, version, 1)[0]
    }

    fn commit(pool: &IdPool, version: u64, nodes: &[u64], edges: &[u64]) {
        pool.finish(
            pool.generation(),
            [&[], &[]],
            Some(([nodes, edges], version)),
        );
    }

    mod id_pool {
        use super::*;

        mod reserve {
            use super::*;

            #[test]
            fn reserve_should_hand_out_free_list_then_new_slots() {
                // GIVEN the pool of the free nodes 4, 2 and no free edge
                with_pool(|pool, graph, view| {
                    // WHEN 4 nodes and an edge are reserved
                    let nodes = [0; 4].map(|_| reserve(pool, graph, view, false, u64::MAX));
                    let edge = reserve(pool, graph, view, true, u64::MAX);

                    // THEN the free list is handed out first, then new slots
                    assert_eq!(nodes, [(4, 0), (2, 0), (6, 0), (7, 0)]);
                    assert_eq!(edge, (2, 0));
                });
            }

            #[test]
            fn reserve_of_many_should_stack_first_on_top() {
                // GIVEN the pool of the free nodes 4, 2
                with_pool(|pool, graph, view| {
                    // WHEN 3 nodes are reserved at once
                    let nodes = reserve_many(pool, graph, view, false, u64::MAX, 3);

                    // THEN the first handed out is on top
                    assert_eq!(nodes, [(6, 0), (2, 0), (4, 0)]);
                });
            }

            #[test]
            fn reserve_after_commit_should_hand_out_last_freed_first() {
                // GIVEN a commit that freed the nodes 3, 5 and the edge 1
                with_pool(|pool, graph, view| {
                    commit(pool, 1, &[3, 5], &[1]);

                    // WHEN 3 nodes and an edge are reserved
                    let nodes = [0; 3].map(|_| reserve(pool, graph, view, false, u64::MAX));
                    let edge = reserve(pool, graph, view, true, u64::MAX);

                    // THEN the slots freed last come first, with their version
                    assert_eq!(nodes, [(5, 1), (3, 1), (4, 0)]);
                    assert_eq!(edge, (1, 1));
                });
            }

            #[test]
            fn reserve_of_older_version_should_skip_slot_freed_after_it() {
                // GIVEN a commit that freed the node 3 in the version 1
                with_pool(|pool, graph, view| {
                    commit(pool, 1, &[3], &[]);

                    // WHEN reserved for the versions 0 and 1
                    // THEN only the version 1 gets the slot 3
                    assert_eq!(reserve(pool, graph, view, false, 0), (4, 0));
                    assert_eq!(reserve(pool, graph, view, false, 1), (3, 1));
                });
            }

            #[test]
            fn reserve_failure_should_fail_keeping_slots_reserved_before() {
                // GIVEN a commit that freed the node 3
                with_pool(|pool, graph, view| {
                    commit(pool, 1, &[3], &[]);
                    let error = DbError::graph(DbErrorType::OutOfBounds, "read failed");
                    let mut reserved = Vec::new();

                    // WHEN reading the free list fails while reserving 2 nodes
                    let result = pool.reserve(
                        false,
                        (1, pool.generation()),
                        2,
                        |_| Err(error.clone()),
                        &mut reserved,
                    );

                    // THEN the reserve fails with the slot 3 reserved and the
                    // free list handed out later
                    assert_eq!(result, Err(error));
                    assert_eq!(reserved, [(3, 1)]);
                    assert_eq!(reserve(pool, graph, view, false, u64::MAX), (4, 0));
                });
            }

            #[test]
            fn reserve_of_older_generation_should_fail() {
                // GIVEN a pool made again
                with_pool(|pool, graph, view| {
                    let generation = pool.generation();
                    pool.reset();
                    pool.init(graph);
                    let mut reserved = Vec::new();
                    let mut next_free = |slot| graph.next_free(view, false, slot);

                    // WHEN a transaction of the older generation reserves
                    let result = pool.reserve(
                        false,
                        (u64::MAX, generation),
                        1,
                        &mut next_free,
                        &mut reserved,
                    );

                    // THEN it fails with the conflict and nothing reserved,
                    // while one of the current generation reserves
                    assert_eq!(result.unwrap_err().ty, DbErrorType::Conflict);
                    assert!(reserved.is_empty());
                    pool.reserve(
                        false,
                        (u64::MAX, pool.generation()),
                        1,
                        &mut next_free,
                        &mut reserved,
                    )
                    .unwrap();
                    assert_eq!(reserved.len(), 1);
                });
            }

            #[test]
            fn reserve_of_pool_not_made_should_fail() {
                // GIVEN a pool reset
                with_pool(|pool, graph, view| {
                    pool.reset();
                    let mut reserved = Vec::new();
                    let mut next_free = |slot| graph.next_free(view, false, slot);

                    // WHEN a transaction reserves
                    let result = pool.reserve(
                        false,
                        (u64::MAX, pool.generation()),
                        1,
                        &mut next_free,
                        &mut reserved,
                    );

                    // THEN it fails with the conflict until the pool is made
                    assert_eq!(result.unwrap_err().ty, DbErrorType::Conflict);
                    assert!(!pool.made());
                    pool.init(graph);
                    pool.reserve(
                        false,
                        (u64::MAX, pool.generation()),
                        1,
                        &mut next_free,
                        &mut reserved,
                    )
                    .unwrap();
                    assert_eq!(reserved.len(), 1);
                });
            }
        }

        mod finish {
            use super::*;

            #[test]
            fn finish_without_commit_should_hand_unused_slots_back() {
                // GIVEN 2 reserved node slots
                with_pool(|pool, graph, view| {
                    let unused = reserve_many(pool, graph, view, false, u64::MAX, 2);

                    // WHEN the transaction ends without a commit
                    pool.finish(pool.generation(), [&unused, &[]], None);

                    // THEN they are reserved again in the same order
                    assert_eq!(reserve_many(pool, graph, view, false, u64::MAX, 2), unused);
                });
            }

            #[test]
            fn finish_with_commit_should_keep_freed_over_unused() {
                // GIVEN an unused node slot 9
                with_pool(|pool, graph, view| {
                    // WHEN the transaction commits having freed the nodes 3, 5
                    // and the edge 1
                    pool.finish(
                        pool.generation(),
                        [&[(9, 0)], &[]],
                        Some(([&[3, 5], &[1]], 1)),
                    );

                    // THEN the freed slots come before the unused one
                    assert_eq!(
                        reserve_many(pool, graph, view, false, u64::MAX, 3),
                        [(9, 0), (3, 1), (5, 1)]
                    );
                    assert_eq!(reserve(pool, graph, view, true, u64::MAX), (1, 1));
                });
            }

            #[test]
            fn finish_with_commit_of_nothing_should_keep_pool() {
                // GIVEN the pool of the free nodes 4, 2
                with_pool(|pool, graph, view| {
                    // WHEN a transaction that took and freed nothing commits
                    commit(pool, 1, &[], &[]);

                    // THEN the pool hands out the same slots
                    assert_eq!(reserve(pool, graph, view, false, u64::MAX), (4, 0));
                });
            }

            #[test]
            fn finish_of_older_generation_should_take_nothing_back() {
                // GIVEN a reserved node slot and the pool made again
                with_pool(|pool, graph, view| {
                    let generation = pool.generation();
                    let reserved = reserve(pool, graph, view, false, u64::MAX);
                    pool.reset();
                    pool.init(graph);

                    // WHEN the transaction of the older generation commits
                    // having freed the node 3
                    pool.finish(generation, [&[reserved], &[]], Some(([&[3], &[]], 1)));

                    // THEN the pool hands out the free list of the graph
                    assert_eq!(
                        reserve_many(pool, graph, view, false, u64::MAX, 3),
                        [(6, 0), (2, 0), (4, 0)]
                    );
                });
            }
        }

        mod reset {
            use super::*;

            #[test]
            fn reset_should_make_pool_again_from_graph() {
                // GIVEN a reserved node slot
                with_pool(|pool, graph, view| {
                    let generation = pool.generation();
                    reserve(pool, graph, view, false, u64::MAX);

                    // WHEN the pool is reset and made again
                    pool.reset();
                    pool.init(graph);

                    // THEN it hands out the free list of the graph in a new
                    // generation
                    assert_eq!(reserve(pool, graph, view, false, u64::MAX), (4, 0));
                    assert_eq!(pool.generation(), generation + 1);
                    assert!(pool.made());
                });
            }
        }
    }
}
