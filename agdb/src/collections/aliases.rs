use crate::DbError;
use crate::DbId;
use crate::collections::graph::GraphHeader;
use crate::collections::table::Kind;
use crate::collections::table::TABLE_SIZE;
use crate::collections::table::TableHeader;
use crate::collections::table::fmix64;
use crate::collections::table::stored_hash;
use crate::collections::value;
use crate::db::db_value::STRING_META_VALUE;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;
use crate::utilities::stable_hash::StableHash;

/// `[hash u64][id i64]`
const ALIASES: Kind = Kind {
    size: 16,
    hash: stored_hash,
};

fn hash(alias: &[u8]) -> u64 {
    fmix64(alias.stable_hash())
}

fn slot_id(slot: &[u8]) -> DbId {
    DbId(u64_at(slot, 1) as i64)
}

fn to_string(alias: &[u8]) -> String {
    String::from_utf8_lossy(alias).into_owned()
}

/// The stored alias of the node `id`, `None` for a node without one, an
/// edge or a missing element.
fn stored(
    view: &impl View,
    graph: &GraphHeader,
    id: DbId,
) -> Result<Option<DbValueIndex>, DbError> {
    Ok(graph.node(view, id)?.and_then(|node| {
        let mut alias = [0_u8; 16];
        alias[..8].copy_from_slice(&node.data[0].to_le_bytes());
        alias[8..].copy_from_slice(&node.data[1].to_le_bytes());
        let alias = DbValueIndex::from_data(alias);
        (alias != DbValueIndex::new()).then_some(alias)
    }))
}

fn set_stored(
    view: &mut impl ViewMut,
    graph: &GraphHeader,
    id: DbId,
    alias: &DbValueIndex,
) -> Result<(), DbError> {
    let alias = alias.data();
    graph.set_data(view, id, 0, &[u64_at(&alias, 0), u64_at(&alias, 1)])
}

/// The aliases of the nodes. A node keeps its alias in the first two
/// fields of its graph data and the table maps the hash of the alias to
/// the node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Aliases(TableHeader);

impl Aliases {
    pub(crate) fn decode(bytes: &[u8]) -> Self {
        Self(TableHeader::decode(bytes))
    }

    pub(crate) fn encode(&self) -> [u8; TABLE_SIZE] {
        self.0.encode()
    }

    pub(crate) fn len(&self) -> u64 {
        self.0.len()
    }

    /// The node with the `alias`.
    pub(crate) fn id(
        &self,
        view: &impl View,
        graph: &GraphHeader,
        alias: &str,
    ) -> Result<Option<DbId>, DbError> {
        let mut probe = self.0.probe(hash(alias.as_bytes()));

        while let Some((_, slot)) = self.0.next(view, &ALIASES, &mut probe)? {
            let id = slot_id(&slot);

            if let Some(index) = stored(view, graph, id)?
                && value::equals_bytes(view, &index, STRING_META_VALUE, alias.as_bytes())?
            {
                return Ok(Some(id));
            }
        }

        Ok(None)
    }

    pub(crate) fn alias(
        &self,
        view: &impl View,
        graph: &GraphHeader,
        id: DbId,
    ) -> Result<Option<String>, DbError> {
        stored(view, graph, id)?
            .map(|index| Ok(to_string(&value::load_bytes(view, &index)?)))
            .transpose()
    }

    /// All aliases with their nodes in the slot order of the table.
    pub(crate) fn aliases(
        &self,
        view: &impl View,
        graph: &GraphHeader,
    ) -> Result<Vec<(String, DbId)>, DbError> {
        let mut aliases = Vec::new();

        for slot in self.0.slots(view, &ALIASES)?.chunks(16) {
            let id = slot_id(slot);
            aliases.extend(self.alias(view, graph, id)?.map(|alias| (alias, id)));
        }

        Ok(aliases)
    }

    /// Sets the `alias` of the existing node `id` replacing its current
    /// alias. The alias must not be used by another node.
    pub(crate) fn insert(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        graph: &GraphHeader,
        id: DbId,
        alias: &str,
    ) -> Result<(), DbError> {
        self.remove(view, alloc, graph, id)?;
        let index = value::store_bytes(view, alloc, STRING_META_VALUE, alias.as_bytes())?;
        set_stored(view, graph, id, &index)?;
        let mut slot = [0_u8; 16];
        slot[..8].copy_from_slice(&hash(alias.as_bytes()).to_le_bytes());
        slot[8..].copy_from_slice(&id.0.to_le_bytes());
        self.0.insert(view, alloc, &ALIASES, &slot)
    }

    /// Removes the alias of the node `id` returning it.
    pub(crate) fn remove(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        graph: &GraphHeader,
        id: DbId,
    ) -> Result<Option<String>, DbError> {
        let Some(index) = stored(view, graph, id)? else {
            return Ok(None);
        };

        let alias = value::load_bytes(view, &index)?;
        let mut probe = self.0.probe(hash(&alias));

        while let Some((at, slot)) = self.0.next(view, &ALIASES, &mut probe)? {
            if slot_id(&slot) == id {
                self.0.remove(view, alloc, &ALIASES, at)?;
                break;
            }
        }

        value::free(view, alloc, &index)?;
        set_stored(view, graph, id, &DbValueIndex::new())?;
        Ok(Some(to_string(&alias)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::graph::tests::allocations as graph_allocations;
    use crate::collections::table::tests::allocation as table_allocation;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::allocation;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::tests::XorShift;
    use crate::storage::root::Layout;
    use std::collections::BTreeMap;

    const LONG: &str = "an alias longer than 15 bytes";

    fn nodes(view: &mut TestView, alloc: &mut Allocator, count: usize) -> (GraphHeader, Vec<DbId>) {
        let mut graph = GraphHeader::new(Layout::default());
        let ids = (0..count)
            .map(|_| graph.insert_node(view, alloc).unwrap())
            .collect();
        (graph, ids)
    }

    fn assert_aliases(
        view: &TestView,
        graph: &GraphHeader,
        aliases: &Aliases,
        model: &BTreeMap<String, DbId>,
    ) {
        let mut all = aliases.aliases(view, graph).unwrap();
        all.sort();
        assert_eq!(
            all,
            model
                .iter()
                .map(|(alias, id)| (alias.clone(), *id))
                .collect::<Vec<_>>()
        );
        assert_eq!(aliases.len(), model.len() as u64);

        for (alias, id) in model {
            assert_eq!(aliases.id(view, graph, alias).unwrap(), Some(*id));
            assert_eq!(
                aliases.alias(view, graph, *id).unwrap().as_ref(),
                Some(alias)
            );
        }
    }

    mod aliases {
        use super::*;

        #[test]
        fn aliases_should_hold_aliases_over_random_operations() {
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let (graph, ids) = nodes(view, alloc, 20);
                    let mut aliases = Aliases::default();
                    let mut model = BTreeMap::<String, DbId>::new();

                    for _ in 0..300 {
                        let id = ids[random.next(ids.len() as u64) as usize];
                        let name = random.next(30);
                        let alias = if name.is_multiple_of(2) {
                            format!("{name}")
                        } else {
                            format!("{LONG}{name}")
                        };
                        let current = model
                            .iter()
                            .find(|(_, node)| **node == id)
                            .map(|(alias, _)| alias.clone());

                        if random.next(3) == 0 {
                            let removed = aliases.remove(view, alloc, &graph, id).unwrap();
                            assert_eq!(removed, current);
                            model.retain(|_, node| *node != id);
                        } else if !model.contains_key(&alias) {
                            aliases.insert(view, alloc, &graph, id, &alias).unwrap();
                            model.retain(|_, node| *node != id);
                            model.insert(alias, id);
                        }

                        assert_aliases(view, &graph, &aliases, &model);
                        assert_eq!(aliases.id(view, &graph, "missing").unwrap(), None);
                    }

                    let mut live = graph_allocations(&graph);
                    live.extend(table_allocation(&aliases.0, &ALIASES));

                    for id in &ids {
                        live.extend(
                            stored(view, &graph, *id)
                                .unwrap()
                                .and_then(|index| allocation(&index)),
                        );
                    }

                    verify(alloc, view, &live);
                });
            }
        }

        #[test]
        fn aliases_failure_should_fail() {
            assert_failures(
                |view, alloc| {
                    let (graph, ids) = nodes(view, alloc, 3);
                    let mut aliases = Aliases::default();
                    aliases.insert(view, alloc, &graph, ids[0], LONG).unwrap();
                    (graph, aliases)
                },
                |view, alloc, state| {
                    let (graph, mut aliases) = state;
                    aliases.insert(view, alloc, &graph, DbId(2), "b")?;
                    aliases.id(view, &graph, LONG)?;
                    aliases.alias(view, &graph, DbId(1))?;
                    aliases.insert(view, alloc, &graph, DbId(1), "a")?;
                    aliases.aliases(view, &graph)?;
                    aliases.insert(view, alloc, &graph, DbId(3), LONG)?;
                    aliases.remove(view, alloc, &graph, DbId(3))?;
                    Ok(())
                },
            );
        }

        mod id {
            use super::*;

            #[test]
            fn id_with_colliding_hash_should_compare_alias() {
                with_view(|view, alloc| {
                    let (graph, ids) = nodes(view, alloc, 2);
                    let mut aliases = Aliases::default();
                    aliases.insert(view, alloc, &graph, ids[0], "a").unwrap();
                    // WHY: the node of "a" now holds "b", as if the two aliases had the same hash
                    let index =
                        value::store_bytes(view, alloc, STRING_META_VALUE, "b".as_bytes()).unwrap();
                    set_stored(view, &graph, ids[0], &index).unwrap();
                    assert_eq!(aliases.id(view, &graph, "a").unwrap(), None);
                    set_stored(view, &graph, ids[0], &DbValueIndex::new()).unwrap();
                    assert_eq!(aliases.id(view, &graph, "a").unwrap(), None);
                });
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_replace_alias_of_node() {
                with_view(|view, alloc| {
                    let (graph, ids) = nodes(view, alloc, 1);
                    let mut aliases = Aliases::default();
                    aliases.insert(view, alloc, &graph, ids[0], "a").unwrap();
                    aliases.insert(view, alloc, &graph, ids[0], LONG).unwrap();
                    let model = BTreeMap::from([(LONG.to_string(), ids[0])]);
                    assert_aliases(view, &graph, &aliases, &model);
                });
            }

            #[test]
            fn insert_to_edge_should_fail() {
                with_view(|view, alloc| {
                    let (mut graph, ids) = nodes(view, alloc, 1);
                    let edge = graph.insert_edge(view, alloc, ids[0], ids[0]).unwrap();
                    let error = Aliases::default()
                        .insert(view, alloc, &graph, edge, "a")
                        .unwrap_err();
                    assert_eq!(error.description, "'-1' is invalid index");
                });
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_alias_of_other_node_with_same_hash_should_keep_it() {
                with_view(|view, alloc| {
                    let (graph, ids) = nodes(view, alloc, 2);
                    let mut aliases = Aliases::default();
                    aliases.insert(view, alloc, &graph, ids[0], "a").unwrap();
                    // WHY: the second node holds "a" without a slot of its own, as if its alias had the hash of "a"
                    let index =
                        value::store_bytes(view, alloc, STRING_META_VALUE, "a".as_bytes()).unwrap();
                    set_stored(view, &graph, ids[1], &index).unwrap();
                    assert_eq!(
                        aliases.remove(view, alloc, &graph, ids[1]).unwrap(),
                        Some("a".to_string())
                    );
                    assert_eq!(aliases.alias(view, &graph, ids[1]).unwrap(), None);
                    assert_eq!(aliases.id(view, &graph, "a").unwrap(), Some(ids[0]));
                });
            }
        }
    }
}
