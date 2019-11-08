use super::comments::Comment;
use crate::schema::{comments, posts};
use crate::AppState;
use actix_web::web::{self, HttpRequest, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use failure::Error;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/posts")
            .route(web::get().to(all_posts))
            .route(web::post().to(new_post)),
    );

    cfg.service(
        web::resource("/posts/{id}")
            .route(web::get().to(get_post_with_id))
            .route(web::patch().to(update_post))
            .route(web::delete().to(delete_post)),
    );

    cfg.service(web::resource("/posts/{id}/comments").route(web::get().to(get_comments_for_post)));
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Post {
    id: i32,
    title: String,
    content: Option<String>,
    published_at: DateTime<Utc>,
    author: i32,
}

#[derive(Deserialize, Debug, AsChangeset)]
#[table_name = "posts"]
struct PostChangeset {
    title: Option<String>,
    content: Option<Option<String>>,
    author: Option<i32>,
}

#[derive(Deserialize, Insertable, Debug)]
#[table_name = "posts"]
struct NewPost {
    title: String,
    content: Option<String>,
    author: i32,
}

fn all_posts(req: HttpRequest) -> Result<Json<Vec<Post>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(posts::table.load(&conn).map(Json)?)
}

fn new_post(req: HttpRequest, new_post: Json<NewPost>) -> Result<Json<Post>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(diesel::insert_into(posts::table)
        .values(new_post.0)
        .get_result(&conn)
        .map(Json)?)
}

fn get_post_with_id(req: HttpRequest, id: web::Path<i32>) -> Result<Json<Post>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(posts::table.find(id.into_inner()).first(&conn).map(Json)?)
}

fn update_post(
    req: HttpRequest,
    id: web::Path<i32>,
    changeset: Json<PostChangeset>,
) -> Result<Json<Post>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(diesel::update(posts::table.find(id.into_inner()))
        .set(changeset.0)
        .get_result(&conn)
        .map(Json)?)
}

fn delete_post(req: HttpRequest, id: web::Path<i32>) -> Result<(), Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    diesel::delete(posts::table.find(id.into_inner())).execute(&conn)?;
    Ok(())
}

fn get_comments_for_post(
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<Json<Vec<Comment>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(comments::table
        .filter(comments::id.eq(id.into_inner()))
        .load(&conn)
        .map(Json)?)
}
