use crate::schema::comments;
use crate::AppState;
use actix_web::web::{self, HttpRequest, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use failure::Error;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/comments")
            .route(web::get().to(all_comments))
            .route(web::post().to(new_comment)),
    );

    cfg.service(
        web::resource("/comments/{id}")
            .route(web::get().to(get_comment_with_id))
            .route(web::patch().to(update_comment))
            .route(web::delete().to(delete_comment)),
    );
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Comment {
    id: i32,
    comment: Option<String>,
    published_at: DateTime<Utc>,
    author: i32,
    post: i32,
}

#[derive(Deserialize, Insertable, Debug)]
#[table_name = "comments"]
struct NewComment {
    comment: Option<String>,
    author: i32,
    post: i32,
}

#[derive(Deserialize, AsChangeset, Debug)]
#[table_name = "comments"]
struct CommentChangeset {
    comment: Option<Option<String>>,
    author: Option<i32>,
    post: Option<i32>,
}

fn all_comments(req: HttpRequest) -> Result<Json<Vec<Comment>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(comments::table.load(&conn).map(Json)?)
}

fn new_comment(req: HttpRequest, new_post: Json<NewComment>) -> Result<Json<Comment>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(diesel::insert_into(comments::table)
        .values(new_post.0)
        .get_result(&conn)
        .map(Json)?)
}

fn get_comment_with_id(req: HttpRequest, id: web::Path<i32>) -> Result<Json<Comment>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(comments::table
        .find(id.into_inner())
        .first(&conn)
        .map(Json)?)
}

fn update_comment(
    req: HttpRequest,
    id: web::Path<i32>,
    changeset: Json<CommentChangeset>,
) -> Result<Json<Comment>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(diesel::update(comments::table.find(id.into_inner()))
        .set(changeset.0)
        .get_result(&conn)
        .map(Json)?)
}

fn delete_comment(req: HttpRequest, id: web::Path<i32>) -> Result<(), Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    diesel::delete(comments::table.find(id.into_inner())).execute(&conn)?;
    Ok(())
}
