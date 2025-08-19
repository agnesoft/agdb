use agdb::CountComparison;
use agdb::DbError;
use agdb::DbMemory;
use agdb::QueryBuilder;

#[derive(Debug)]
struct UserDb {
    name: String,
    #[allow(dead_code)]
    role: String,
}

fn main() -> Result<(), DbError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Inserts root node for databases and the user.
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["user", "dbs"])
            .query(),
    )?;

    // Create two databases.
    let dbs = db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("name", "db1").into()], [("name", "db2").into()]])
            .query(),
    )?;

    // Attach the databases to the user with the roles "admin" and "read" respectively.
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("user")
            .to(dbs)
            .values([[("role", "admin").into()], [("role", "read").into()]])
            .query(),
    )?;

    // Since there are no native joins in agdb we emulate it by selecting all the data
    // we need and then join them when iterating over the returned elements. It is important
    // that the elements follow expected order = in this case an edge is always followed by a node.
    // We have achived that by specifying the depth first search in the query. On each edge we
    // create the joined element and on each node we fill in the remaining data.
    let mut user_dbs: Vec<UserDb> = vec![];
    db.exec(
        QueryBuilder::select()
            .search()
            .depth_first()
            .from("user")
            .where_()
            .keys("role")
            .or()
            .keys("name")
            .query(),
    )?
    .elements
    .into_iter()
    .for_each(|e| {
        if e.id.0 < 0 {
            // handle the edge
            user_dbs.push(UserDb {
                role: e.values[0].value.to_string(),
                name: String::new(),
            });
        } else {
            // handle the node
            user_dbs.last_mut().unwrap().name = e.values[0].value.to_string();
        }
    });

    println!("{user_dbs:?}");

    // If the data is sparse, we need other processing or require better control over the joining
    // you should use a transaction and split the steps into subqueries. The transaction makes
    // sure your data is not modified under your hands. Using the trnsaction and multiple queries
    // lets you do additional processing, filtering or mapping of the data. It is worth noting that
    // your specialized "join" will always be faster than a generic one the database could offer, particularly
    // when using Rust. Joins on the database side makes sense when you are using an interpreted language
    // which is significnatly slower.
    let user_dbs = db.transaction(|t| -> Result<Vec<UserDb>, DbError> {
        let db_names = t.exec(
            QueryBuilder::select()
                .search()
                .from("user")
                .where_()
                .distance(CountComparison::Equal(2))
                .and()
                .keys("name")
                .query(),
        )?;

        let mut dbs = vec![];

        for db_name in db_names.elements {
            let role = t.exec(
                QueryBuilder::select()
                    .search()
                    .to(db_name.id)
                    .limit(1)
                    .where_()
                    .keys("role")
                    .query(),
            )?;

            dbs.push(UserDb {
                name: db_name.values[0].value.to_string(),
                role: role.elements[0].values[0].value.to_string(),
            });
        }

        Ok(dbs)
    })?;

    println!("{user_dbs:?}");

    Ok(())
}
