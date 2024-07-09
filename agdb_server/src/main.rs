mod api;
mod app;
mod cluster;
mod config;
mod db_pool;
mod error_code;
mod logger;
mod password;
mod routes;
mod server_error;
mod server_state;
mod user_id;
mod utilities;

use crate::db_pool::DbPool;
use server_error::ServerResult;
use tokio::sync::broadcast;
use tracing::Level;

#[tokio::main]
async fn main() -> ServerResult {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);
    let config = config::new()?;
    let cluster = cluster::new(&config)?;
    let db_pool = DbPool::new(&config).await?;
    let app = app::app(
        config.clone(),
        shutdown_sender.clone(),
        db_pool,
        cluster.clone(),
    );
    tracing::info!("Listening at {}", config.bind);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(cluster::start_with_shutdown(cluster, shutdown_receiver))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::api::Api;
    use agdb::Comparison;
    use agdb::CountComparison;
    use agdb::DbKeyOrder;
    use agdb::QueryBuilder;
    use agdb::QueryId;
    use agdb::QueryType;
    use agdb::UserValue;
    use std::fs::File;
    use std::io::Write;
    use utoipa::OpenApi;

    macro_rules! queries {
        ($($x:expr),+ $(,)?) => {
            {
                let mut vec: Vec<(String, QueryType)> = Vec::new();
                $(
                    {
                        let mut as_string = stringify!($x).to_string();
                        as_string.retain(|c| !c.is_whitespace());
                        vec.push((as_string, $x.into()));
                    }
                )*
                vec
            }
        };
    }

    #[derive(Default, UserValue)]
    struct T {
        db_id: Option<QueryId>,
        value1: String,
        value2: u64,
    }

    #[test]
    fn generate_openapi_schema() {
        let schema = Api::openapi().to_pretty_json().unwrap();
        let mut file = File::create("openapi/schema.json").unwrap();
        file.write_all(schema.as_bytes()).unwrap();
    }

    #[test]
    fn generate_test_suite() {
        #[rustfmt::skip]
        let queries = queries![
QueryBuilder::insert().aliases("a").ids(1).query(),
QueryBuilder::insert().aliases("a").ids("b").query(),
QueryBuilder::insert().aliases(vec!["a", "b"]).ids(vec![1, 2]).query(),
QueryBuilder::insert().edges().from(1).to(2).query(),
QueryBuilder::insert().edges().from("a").to("b").query(),
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).query(),
QueryBuilder::insert().edges().from(vec![1, 2]).to(vec![2, 3]).query(),
QueryBuilder::insert().edges().from(vec![1, 2]).to(vec![2, 3]).each().query(),
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query(),
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().edges().ids(-3).from(1).to(2).query(),
QueryBuilder::insert().edges().ids(vec![-3, -4]).from(1).to(2).query(),
QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query(),
QueryBuilder::insert().index("key").query(),
QueryBuilder::insert().nodes().count(2).query(),
QueryBuilder::insert().nodes().count(2).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).query(),
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query(),
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query(),
QueryBuilder::insert().nodes().ids(1).count(1).query(),
QueryBuilder::insert().nodes().ids(vec![1, 2]).count(1).query(),
QueryBuilder::insert().nodes().ids("a").count(1).query(),
QueryBuilder::insert().nodes().ids(vec!["a", "b"]).count(1).query(),
QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query(),
QueryBuilder::insert().element(&T::default()).query(),
QueryBuilder::insert().elements(&[T::default(), T::default()]).query(),
QueryBuilder::insert().values(vec![vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(vec![1, 2]).query(),
QueryBuilder::insert().values(vec![vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).ids(vec![1, 2]).query(),
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::remove().aliases("a").query(),
QueryBuilder::remove().aliases(vec!["a", "b"]).query(),
QueryBuilder::remove().ids(1).query(),
QueryBuilder::remove().ids("a").query(),
QueryBuilder::remove().ids(vec![1, 2]).query(),
QueryBuilder::remove().ids(vec!["a", "b"]).query(),
QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::remove().index("key").query(),
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).ids(vec![1, 2]).query(),
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::select().aliases().ids(vec![1, 2]).query(),
QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().aliases().query(),
QueryBuilder::select().edge_count().ids(vec![1, 2]).query(),
QueryBuilder::select().edge_count_from().ids(vec![1, 2]).query(),
QueryBuilder::select().edge_count_to().ids(vec![1, 2]).query(),
QueryBuilder::select().ids("a").query(),
QueryBuilder::select().ids(vec![1, 2]).query(),
QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().indexes().query(),
QueryBuilder::select().keys().ids("a").query(),
QueryBuilder::select().keys().ids(vec![1, 2]).query(),
QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().key_count().ids("a").query(),
QueryBuilder::select().key_count().ids(vec![1, 2]).query(),
QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().node_count().query(),
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids("a").query(),
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids(vec![1, 2]).query(),
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::search().from("a").query(),
QueryBuilder::search().to(1).query(), 
QueryBuilder::search().from("a").to("b").query(), 
QueryBuilder::search().breadth_first().from("a").query(), 
QueryBuilder::search().depth_first().from("a").query(),
QueryBuilder::search().elements().query(),
QueryBuilder::search().index("age").value(20).query(), 
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("age".into()), DbKeyOrder::Asc("name".into())]).query(),
QueryBuilder::search().from(1).offset(10).query(),
QueryBuilder::search().from(1).limit(5).query(),
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).query(),
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).limit(5).query(),
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query(),
QueryBuilder::search().from(1).offset(10).limit(5).query(),
QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query(),
QueryBuilder::search().from(1).where_().edge().query(),
QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query(),
QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query(),
QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query(),
QueryBuilder::search().from(1).where_().node().query(),
QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query(),
QueryBuilder::search().from(1).where_().keys(vec!["k1".into(), "k2".into()]).query(),
QueryBuilder::search().from(1).where_().not().keys(vec!["k1".into(), "k2".into()]).query(),
QueryBuilder::search().from(1).where_().ids(vec![1, 2]).query(),
QueryBuilder::search().from(1).where_().beyond().keys(vec!["k".into()]).query(),
QueryBuilder::search().from(1).where_().not().ids(vec![1, 2]).query(),
QueryBuilder::search().from(1).where_().not_beyond().ids("a").query(),
QueryBuilder::search().from(1).where_().node().or().edge().query(),
QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains((vec![1, 2]).into())).end_where().query(),
        ];

        serde_json::to_writer_pretty(File::create("openapi/test_queries.json").unwrap(), &queries)
            .unwrap();
    }
}
