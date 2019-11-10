#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;
use wundergraph::scalar::WundergraphScalarValue;

mod graphql;
mod model;
mod pagination;
#[allow(unused_imports)]
mod schema;
#[macro_use]
mod diesel_ext;

use self::graphql::{Mutation, Query};

#[derive(Debug, StructOpt)]
#[structopt(name = "rustfest")]
struct Opt {
    #[structopt(short = "u", long = "db-url")]
    database_url: String,
    #[structopt(short = "s", long = "socket", default_value = "127.0.0.1:8000")]
    socket: String,
}

pub type Schema =
    juniper::RootNode<'static, Query<PgConnection>, Mutation<PgConnection>, WundergraphScalarValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest<WundergraphScalarValue>);

#[derive(Clone)]
struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
    schema: Arc<Schema>,
}

fn graphql(
    web::Json(GraphQLData(data)): web::Json<GraphQLData>,
    st: web::Data<AppState>,
) -> Result<HttpResponse, failure::Error> {
    let ctx = st.get_ref().pool.get()?;
    let res = data.execute(&st.get_ref().schema, &ctx);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn main() {
    let opt = Opt::from_args();
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = ConnectionManager::<PgConnection>::new(opt.database_url);
    let pool = Pool::builder().build(manager).expect("Failed to init pool");

    diesel_migrations::run_pending_migrations(&pool.get().expect("Failed to get db connection"))
        .expect("Failed to run migrations");

    let query = Query::<PgConnection>::default();
    let mutation = Mutation::<PgConnection>::default();
    let schema = Arc::new(Schema::new(query, mutation));
    let data = AppState { pool, schema };

    let url = opt.socket;

    println!("Started http server: http://{}", url);

    HttpServer::new(move || {
        App::new()
            .configure(model::posts::config)
            .configure(model::users::config)
            .configure(model::comments::config)
            .route("/graphiql", web::get().to(graphiql))
            .route("/graphql", web::get().to(graphql))
            .route("/graphql", web::post().to(graphql))
            .data(data.clone())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| {
                HttpResponse::Found()
                    .header("location", "/graphiql")
                    .finish()
            }))
    })
    .bind(&url)
    .expect("Failed to start server")
    .run()
    .unwrap();
}
