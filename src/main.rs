#[macro_use]
extern crate diesel;

use actix_web::{middleware, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use structopt::StructOpt;

mod model;
mod schema;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustfest")]
struct Opt {
    #[structopt(short = "u", long = "db-url")]
    database_url: String,
    #[structopt(short = "s", long = "socket", default_value = "127.0.0.1:8000")]
    socket: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
}

fn main() {
    let opt = Opt::from_args();
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = ConnectionManager::<PgConnection>::new(opt.database_url);
    let pool = Pool::builder().build(manager).expect("Failed to init pool");

    diesel_migrations::run_pending_migrations(&pool.get().expect("Failed to get db connection"))
        .expect("Failed to run migrations");

    let data = AppState { pool };

    let url = opt.socket;

    println!("Started http server: http://{}", url);

    HttpServer::new(move || {
        App::new()
            .configure(model::posts::config)
            .configure(model::users::config)
            .configure(model::comments::config)
            .data(data.clone())
            .wrap(middleware::Logger::default())
    })
    .bind(&url)
    .expect("Failed to start server")
    .run()
    .unwrap();
}
