mod test_db;

use agdb::DbKeyOrder;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn search_elements() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_ids(QueryBuilder::search().elements().query(), &[1, 2, 3]);
}

#[test]
fn search_elements_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(QueryBuilder::search().elements().query(), &[1, 2, 3, -4]);
}

#[test]
fn search_elements_removed_node() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_mut(QueryBuilder::remove().ids(2).query(), -1);
    db.exec_ids(QueryBuilder::search().elements().query(), &[1, 3, -4]);
}

#[test]
fn search_elements_removed_node_inserted_edge() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_mut(QueryBuilder::remove().ids(2).query(), -1);
    db.exec_mut(QueryBuilder::insert().edges().from(3).to(1).query(), 1);
    db.exec_ids(QueryBuilder::search().elements().query(), &[1, -2, 3, -4]);
}

#[test]
fn search_elements_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(QueryBuilder::search().elements().limit(2).query(), &[1, 2]);
}

#[test]
fn search_elements_offset() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(
        QueryBuilder::search().elements().offset(2).query(),
        &[3, -4],
    );
}

#[test]
fn search_elements_offset_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(
        QueryBuilder::search().elements().offset(1).limit(2).query(),
        &[2, 3],
    );
}

#[test]
fn search_elements_order_by() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                [("age", 20).into()],
                [("age", 15).into()],
                [("age", 30).into()],
            ])
            .query(),
        3,
    );
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(
        QueryBuilder::search()
            .elements()
            .order_by([DbKeyOrder::Asc("age".into())])
            .query(),
        &[2, 1, 3, -4],
    );
}

#[test]
fn search_elements_condition() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_ids(
        QueryBuilder::search()
            .elements()
            .where_()
            .not_beyond()
            .ids(2)
            .and()
            .edge()
            .query(),
        &[-4],
    );
}
