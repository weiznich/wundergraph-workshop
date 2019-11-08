use super::comments::Comment;
use super::posts::Post;
use crate::schema::{comments, posts, users};
use crate::AppState;
use actix_web::web::{self, HttpRequest, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use failure::Error;
use juniper::GraphQLInputObject;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(all_users))
            .route(web::post().to(new_user)),
    );

    cfg.service(
        web::resource("/users/{id}")
            .route(web::get().to(get_user_with_id))
            .route(web::patch().to(update_user))
            .route(web::delete().to(delete_user)),
    );

    cfg.service(web::resource("/users/{id}/posts").route(web::get().to(get_posts_for_user)));
    cfg.service(web::resource("/users/{id}/comments").route(web::get().to(get_comments_for_user)));
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct User {
    id: i32,
    name: String,
    joined_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Insertable, GraphQLInputObject)]
#[table_name = "users"]
pub struct NewUser {
    name: String,
}

#[derive(Deserialize, Debug, AsChangeset, GraphQLInputObject)]
#[table_name = "users"]
pub struct UserChangeset {
    name: Option<String>,
}

fn all_users(req: HttpRequest) -> Result<Json<Vec<User>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(users::table.load(&conn).map(Json)?)
}

fn new_user(req: HttpRequest, new_user: Json<NewUser>) -> Result<Json<User>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(diesel::insert_into(users::table)
        .values(new_user.0)
        .get_result(&conn)
        .map(Json)?)
}

fn get_user_with_id(req: HttpRequest, id: web::Path<i32>) -> Result<Json<User>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(users::table.find(id.into_inner()).first(&conn).map(Json)?)
}

fn update_user(
    req: HttpRequest,
    id: web::Path<i32>,
    changeset: Json<UserChangeset>,
) -> Result<Json<User>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(diesel::update(users::table.find(id.into_inner()))
        .set(changeset.0)
        .get_result(&conn)
        .map(Json)?)
}

fn delete_user(req: HttpRequest, id: web::Path<i32>) -> Result<(), Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    diesel::delete(users::table.find(id.into_inner())).execute(&conn)?;
    Ok(())
}

fn get_posts_for_user(req: HttpRequest, id: web::Path<i32>) -> Result<Json<Vec<Post>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(posts::table
        .filter(posts::author.eq(id.into_inner()))
        .load(&conn)
        .map(Json)?)
}

fn get_comments_for_user(
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<Json<Vec<Comment>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    Ok(comments::table
        .filter(comments::author.eq(id.into_inner()))
        .load(&conn)
        .map(Json)?)
}
