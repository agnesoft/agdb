use crate::DbElement;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbImpl;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryBuilder;
use crate::QueryResult;
use crate::StorageData;
use crate::TransactionMut;
use crate::storage::pages::tests::XorShift;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const LONG: &str = "longer than the 15 inline bytes";
const ALIASES: [&str; 4] = ["a", "b", "c", LONG];
const MAX_ELEMENTS: usize = 40;

/// A mutation of the random query sequences.
#[derive(Clone, Debug)]
pub(super) enum Op {
    InsertNodes {
        aliases: Vec<String>,
        values: Vec<Vec<DbKeyValue>>,
    },
    InsertEdge {
        from: DbId,
        to: DbId,
        values: Vec<DbKeyValue>,
    },
    InsertValues {
        id: DbId,
        values: Vec<DbKeyValue>,
    },
    RemoveValues {
        id: DbId,
        keys: Vec<DbValue>,
    },
    Remove {
        id: DbId,
    },
    InsertAlias {
        id: DbId,
        alias: String,
    },
    RemoveAlias {
        alias: String,
    },
    InsertIndex {
        key: DbValue,
    },
    RemoveIndex {
        key: DbValue,
    },
    /// The ops in one transaction that then fails.
    Failing(Vec<Op>),
}

impl Op {
    pub(super) fn run<S: StorageData>(&self, db: &mut DbImpl<S>) -> Result<QueryResult, DbError> {
        db.transaction_mut(|t| self.exec(t))
    }

    fn exec<S: StorageData>(&self, t: &mut TransactionMut<S>) -> Result<QueryResult, DbError> {
        match self {
            Op::InsertNodes { aliases, values } => t.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(aliases.clone())
                    .values(values.clone())
                    .query(),
            ),
            Op::InsertEdge { from, to, values } => t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(*from)
                    .to(*to)
                    .values(vec![values.clone()])
                    .query(),
            ),
            Op::InsertValues { id, values } => t.exec_mut(
                QueryBuilder::insert()
                    .values(vec![values.clone()])
                    .ids(*id)
                    .query(),
            ),
            Op::RemoveValues { id, keys } => {
                t.exec_mut(QueryBuilder::remove().values(keys.clone()).ids(*id).query())
            }
            Op::Remove { id } => t.exec_mut(QueryBuilder::remove().ids(*id).query()),
            Op::InsertAlias { id, alias } => t.exec_mut(
                QueryBuilder::insert()
                    .aliases(alias.clone())
                    .ids(*id)
                    .query(),
            ),
            Op::RemoveAlias { alias } => {
                t.exec_mut(QueryBuilder::remove().aliases(alias.clone()).query())
            }
            Op::InsertIndex { key } => {
                t.exec_mut(QueryBuilder::insert().index(key.clone()).query())
            }
            Op::RemoveIndex { key } => {
                t.exec_mut(QueryBuilder::remove().index(key.clone()).query())
            }
            Op::Failing(ops) => {
                for op in ops {
                    let _ = op.exec(t);
                }

                Err(DbError::db(DbErrorType::NotAllowed, "failing transaction"))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Element {
    from: DbId,
    to: DbId,
    values: Vec<DbKeyValue>,
}

/// What the database must hold after the ops applied to it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct TestModel {
    elements: BTreeMap<DbId, Element>,
    aliases: BTreeMap<String, DbId>,
    indexes: BTreeSet<DbValue>,
}

/// The database content in a form independent of its storage order.
#[derive(Debug, PartialEq)]
pub(super) struct Dump {
    elements: Vec<DbElement>,
    aliases: Vec<(String, DbId)>,
    indexes: Vec<DbKeyValue>,
    links: Vec<(DbId, Vec<DbId>, Vec<DbId>)>,
    edge_counts: Vec<(DbId, DbValue, DbValue)>,
    found: Vec<(DbValue, DbValue, Option<Vec<DbId>>)>,
    counts: (u64, u64),
}

fn key(random: &mut XorShift) -> DbValue {
    match random.next(5) {
        0 => "k".into(),
        1 => "j".into(),
        2 => "l".into(),
        3 => LONG.into(),
        _ => 7_u64.into(),
    }
}

fn value(random: &mut XorShift) -> DbValue {
    match random.next(8) {
        0 => 1_i64.into(),
        1 => 2_u64.into(),
        2 => "v".into(),
        3 => LONG.into(),
        4 => vec![1.5_f64, 2.5].into(),
        5 => vec!["x".to_string(), LONG.to_string()].into(),
        6 => DbValue::None,
        _ => 3.5_f64.into(),
    }
}

/// Up to `max` distinct keys.
fn keys(random: &mut XorShift, max: u64) -> Vec<DbValue> {
    let mut keys = Vec::new();

    for _ in 0..random.next(max + 1) {
        let key = key(random);

        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    keys
}

fn values(random: &mut XorShift, max: u64) -> Vec<DbKeyValue> {
    keys(random, max)
        .into_iter()
        .map(|key| DbKeyValue {
            key,
            value: value(random),
        })
        .collect()
}

fn alias(random: &mut XorShift) -> String {
    ALIASES[random.next(ALIASES.len() as u64) as usize].to_string()
}

fn set(values: &mut Vec<DbKeyValue>, key_value: &DbKeyValue) {
    if let Some(old) = values.iter_mut().find(|old| old.key == key_value.key) {
        old.value = key_value.value.clone();
    } else {
        values.push(key_value.clone());
    }
}

fn sorted(mut ids: Vec<DbId>) -> Vec<DbId> {
    ids.sort();
    ids
}

impl TestModel {
    pub(super) fn op(&self, random: &mut XorShift) -> Op {
        if random.next(20) == 0 {
            Op::Failing(
                (0..=random.next(3))
                    .map(|_| self.simple_op(random))
                    .collect(),
            )
        } else {
            self.simple_op(random)
        }
    }

    fn simple_op(&self, random: &mut XorShift) -> Op {
        let nodes: Vec<DbId> = self
            .elements
            .keys()
            .filter(|id| 0 < id.0)
            .copied()
            .collect();
        let full = MAX_ELEMENTS < self.elements.len();

        match random.next(19) {
            0..=3 if !full || nodes.is_empty() => {
                let count = 1 + random.next(3);
                let mut aliases = Vec::new();

                for _ in 0..random.next(count + 1) {
                    let alias = alias(random);

                    if !aliases.contains(&alias) {
                        aliases.push(alias);
                    }
                }

                Op::InsertNodes {
                    aliases,
                    values: (0..count).map(|_| values(random, 3)).collect(),
                }
            }
            4..=7 if !full && !nodes.is_empty() => Op::InsertEdge {
                from: nodes[random.next(nodes.len() as u64) as usize],
                to: nodes[random.next(nodes.len() as u64) as usize],
                values: values(random, 2),
            },
            8..=10 => Op::InsertValues {
                id: self.id(random),
                values: values(random, 3),
            },
            11 => Op::RemoveValues {
                id: self.id(random),
                keys: keys(random, 2),
            },
            12 => Op::InsertAlias {
                id: self.id(random),
                alias: alias(random),
            },
            13 => Op::RemoveAlias {
                alias: alias(random),
            },
            14 => Op::InsertIndex { key: key(random) },
            15 => Op::RemoveIndex { key: key(random) },
            _ => Op::Remove {
                id: self.id(random),
            },
        }
    }

    /// An element, or one beyond them all now and then.
    fn id(&self, random: &mut XorShift) -> DbId {
        let len = self.elements.len() as u64;

        if len == 0 || random.next(10) == 0 {
            let beyond = self.elements.keys().map(|id| id.0.abs()).max().unwrap_or(0) + 1;
            return DbId(if random.next(2) == 0 { beyond } else { -beyond });
        }

        *self.elements.keys().nth(random.next(len) as usize).unwrap()
    }

    /// The `op` fails as a whole, it changes nothing.
    pub(super) fn fails(&self, op: &Op) -> bool {
        match op {
            Op::InsertValues { id, .. } | Op::RemoveValues { id, .. } => {
                !self.elements.contains_key(id)
            }
            Op::InsertAlias { id, .. } => id.0 < 0 || !self.elements.contains_key(id),
            Op::InsertIndex { key } => self.indexes.contains(key),
            Op::Failing(_) => true,
            _ => false,
        }
    }

    /// The model after the `op` with the `result` it returned.
    pub(super) fn apply(&self, op: &Op, result: &Result<QueryResult, DbError>) -> TestModel {
        if self.fails(op) {
            assert!(result.is_err(), "{op:?} should fail: {result:?}");
            return self.clone();
        }

        let result = result
            .as_ref()
            .unwrap_or_else(|error| panic!("{op:?}: {error}"));
        let mut next = self.clone();

        match op {
            Op::InsertNodes { aliases, values } => {
                assert_eq!(result.elements.len(), values.len(), "{op:?}");

                for (index, (element, values)) in result.elements.iter().zip(values).enumerate() {
                    let alias = aliases.get(index);

                    if let Some(owner) = alias.and_then(|alias| next.aliases.get(alias)) {
                        assert_eq!(element.id, *owner, "{op:?}");
                        let element = next.elements.get_mut(owner).unwrap();
                        values
                            .iter()
                            .for_each(|value| set(&mut element.values, value));
                    } else {
                        next.insert(element.id, DbId(0), DbId(0), values);

                        if let Some(alias) = alias {
                            next.aliases.insert(alias.clone(), element.id);
                        }
                    }
                }
            }
            Op::InsertEdge { from, to, values } => {
                assert!(result.elements[0].id.0 < 0, "{op:?}");
                next.insert(result.elements[0].id, *from, *to, values);
            }
            Op::InsertValues { id, values } => {
                let element = next.elements.get_mut(id).unwrap();
                values
                    .iter()
                    .for_each(|value| set(&mut element.values, value));
            }
            Op::RemoveValues { id, keys } => {
                let element = next.elements.get_mut(id).unwrap();
                element.values.retain(|value| !keys.contains(&value.key));
            }
            Op::Remove { id } => next.remove(*id),
            Op::InsertAlias { id, alias } => {
                next.aliases.retain(|_, owner| owner != id);
                next.aliases.insert(alias.clone(), *id);
            }
            Op::RemoveAlias { alias } => {
                next.aliases.remove(alias);
            }
            Op::InsertIndex { key } => {
                next.indexes.insert(key.clone());
            }
            Op::RemoveIndex { key } => {
                next.indexes.remove(key);
            }
            Op::Failing(_) => unreachable!(),
        }

        next
    }

    fn insert(&mut self, id: DbId, from: DbId, to: DbId, values: &[DbKeyValue]) {
        let element = Element {
            from,
            to,
            values: values.to_vec(),
        };
        assert!(self.elements.insert(id, element).is_none(), "{id:?} taken");
    }

    fn remove(&mut self, id: DbId) {
        if self.elements.remove(&id).is_some() && 0 < id.0 {
            self.elements
                .retain(|edge, element| 0 < edge.0 || (element.from != id && element.to != id));
            self.aliases.retain(|_, owner| *owner != id);
        }
    }

    fn edges(&self, node: DbId, from: bool) -> Vec<DbId> {
        self.elements
            .iter()
            .filter(|(id, element)| {
                id.0 < 0 && node == if from { element.from } else { element.to }
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// The index searches of the dump: each value of an indexed key and
    /// one that no element has.
    fn lookups(&self) -> Vec<(DbValue, DbValue)> {
        let mut lookups = BTreeSet::new();

        for key in &self.indexes {
            lookups.insert((key.clone(), DbValue::from("absent")));

            for element in self.elements.values() {
                for value in element.values.iter().filter(|value| &value.key == key) {
                    lookups.insert((key.clone(), value.value.clone()));
                }
            }
        }

        lookups.into_iter().collect()
    }

    pub(super) fn dump(&self) -> Dump {
        let nodes: Vec<DbId> = self
            .elements
            .keys()
            .filter(|id| 0 < id.0)
            .copied()
            .collect();
        let count = |key: &DbValue, value: Option<&DbValue>| {
            self.elements
                .iter()
                .filter(|(_, element)| {
                    element
                        .values
                        .iter()
                        .any(|kv| &kv.key == key && value.is_none_or(|value| &kv.value == value))
                })
                .map(|(id, _)| *id)
                .collect::<Vec<DbId>>()
        };

        Dump {
            elements: self
                .elements
                .iter()
                .map(|(id, element)| DbElement {
                    id: *id,
                    from: element.from,
                    to: element.to,
                    values: element.values.clone(),
                })
                .collect(),
            aliases: self
                .aliases
                .iter()
                .map(|(alias, id)| (alias.clone(), *id))
                .collect(),
            indexes: self
                .indexes
                .iter()
                .map(|key| DbKeyValue {
                    key: key.clone(),
                    value: (count(key, None).len() as u64).into(),
                })
                .collect(),
            links: nodes
                .iter()
                .map(|id| (*id, self.edges(*id, true), self.edges(*id, false)))
                .collect(),
            edge_counts: nodes
                .iter()
                .map(|id| {
                    (
                        *id,
                        (self.edges(*id, true).len() as u64).into(),
                        (self.edges(*id, false).len() as u64).into(),
                    )
                })
                .collect(),
            found: self
                .lookups()
                .into_iter()
                .map(|(key, value)| {
                    let ids = count(&key, Some(&value));
                    (key, value, Some(ids))
                })
                .collect(),
            counts: (
                nodes.len() as u64,
                (self.elements.len() - nodes.len()) as u64,
            ),
        }
    }
}

impl Dump {
    /// Reads the `db` with the index searches of the `model`.
    pub(super) fn of<S: StorageData>(db: &DbImpl<S>, model: &TestModel) -> Dump {
        let mut elements = db
            .exec(QueryBuilder::select().search().elements().query())
            .unwrap()
            .elements;
        elements.sort_by_key(|element| element.id);
        elements
            .iter_mut()
            .filter(|element| 0 < element.id.0)
            .for_each(|node| {
                node.from = DbId(0);
                node.to = DbId(0);
            });
        let nodes: Vec<DbId> = elements
            .iter()
            .map(|element| element.id)
            .filter(|id| 0 < id.0)
            .collect();

        let mut aliases: Vec<(String, DbId)> = db
            .exec(QueryBuilder::select().aliases().query())
            .unwrap()
            .elements
            .into_iter()
            .map(|element| (element.values[0].value.to_string(), element.id))
            .collect();
        aliases.sort();

        let mut indexes = db
            .exec(QueryBuilder::select().indexes().query())
            .unwrap()
            .elements[0]
            .values
            .clone();
        indexes.sort_by(|a, b| a.key.cmp(&b.key));

        let edges =
            |found: QueryResult| sorted(found.ids().into_iter().filter(|id| id.0 < 0).collect());
        let links = nodes
            .iter()
            .map(|id| {
                let from = db.exec(
                    QueryBuilder::search()
                        .from(*id)
                        .where_()
                        .beyond()
                        .distance_less_than(2)
                        .query(),
                );
                let to = db.exec(
                    QueryBuilder::search()
                        .to(*id)
                        .where_()
                        .beyond()
                        .distance_less_than(2)
                        .query(),
                );
                (*id, edges(from.unwrap()), edges(to.unwrap()))
            })
            .collect();

        let from = db
            .exec(
                QueryBuilder::select()
                    .edge_count_from()
                    .ids(nodes.clone())
                    .query(),
            )
            .unwrap()
            .elements;
        let to = db
            .exec(
                QueryBuilder::select()
                    .edge_count_to()
                    .ids(nodes.clone())
                    .query(),
            )
            .unwrap()
            .elements;
        let edge_counts = from
            .into_iter()
            .zip(to)
            .map(|(from, to)| {
                (
                    from.id,
                    from.values[0].value.clone(),
                    to.values[0].value.clone(),
                )
            })
            .collect();

        let found = model
            .lookups()
            .into_iter()
            .map(|(key, value)| {
                let ids = db
                    .exec(
                        QueryBuilder::search()
                            .index(key.clone())
                            .value(value.clone())
                            .query(),
                    )
                    .map(|found| sorted(found.ids()))
                    .ok();
                (key, value, ids)
            })
            .collect();

        let counts = (
            db.exec(QueryBuilder::select().node_count().query())
                .unwrap()
                .result,
            db.exec(QueryBuilder::select().edge_count().query())
                .unwrap()
                .result,
        );

        Dump {
            elements,
            aliases,
            indexes,
            links,
            edge_counts,
            found,
            counts,
        }
    }
}

/// A random op sequence of a seed and the model of its outcome.
pub(super) struct Run {
    seed: u64,
    step: u64,
    random: XorShift,
    pub(super) model: TestModel,
}

impl Run {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            seed,
            step: 0,
            random: XorShift(seed),
            model: TestModel::default(),
        }
    }

    pub(super) fn op(&mut self) -> Op {
        self.step += 1;
        self.model.op(&mut self.random)
    }

    pub(super) fn apply(&mut self, op: &Op, result: &Result<QueryResult, DbError>) {
        self.model = self.model.apply(op, result);
    }

    /// Asserts that the `db` holds the model.
    pub(super) fn check<S: StorageData>(&self, db: &DbImpl<S>, context: &str) {
        assert_eq!(
            Dump::of(db, &self.model),
            self.model.dump(),
            "seed {}, step {}: {context}",
            self.seed,
            self.step
        );
    }

    /// Runs the next op on the `db` and checks it holds the model after it.
    pub(super) fn step<S: StorageData>(&mut self, db: &mut DbImpl<S>) {
        let op = self.op();
        let result = op.run(db);
        self.apply(&op, &result);
        self.check(db, &format!("{op:?}"));
    }
}
