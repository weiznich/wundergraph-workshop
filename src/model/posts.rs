use super::comments::Comment;
use crate::pagination::{Paginate, DEFAULT_PER_PAGE};
use crate::schema::{comments, posts};
use crate::AppState;
use actix_web::web::{self, HttpRequest, Json};
use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{self, ToSql};
use failure::Error;
use juniper::{GraphQLEnum, GraphQLInputObject};
use serde::{Deserialize, Serialize};
use std::io::Write;
use wundergraph::query_builder::types::WundergraphValue;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/posts")
            .route(web::get().to(get_posts_with_query))
            .route(web::post().to(new_post)),
    );

    cfg.service(
        web::resource("/posts/{id}")
            .route(web::get().to(get_post_with_id))
            .route(web::patch().to(update_post))
            .route(web::delete().to(delete_post)),
    );

    cfg.service(web::resource("/posts/{id}/comments").route(web::get().to(get_comments_for_post)));

    cfg.service(web::resource("/posts/page/{page_number}").route(web::get().to(paginated_posts)));
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Post {
    id: i32,
    title: String,
    content: Option<String>,
    published_at: DateTime<Utc>,
    author: i32,
    post_state: PostState,
    version_start: i32,
    version_end: Option<i32>,
}

#[derive(Deserialize, Debug, AsChangeset, GraphQLInputObject)]
#[table_name = "posts"]
pub struct PostChangeset {
    title: Option<String>,
    content: Option<Option<String>>,
    author: Option<i32>,
}

#[derive(Deserialize, Insertable, Debug, GraphQLInputObject)]
#[table_name = "posts"]
pub struct NewPost {
    title: String,
    content: Option<String>,
    author: i32,
}

#[derive(Debug, Clone, Copy, SqlType, QueryId)]
#[allow(non_camel_case_types)]
#[postgres(type_name = "post_state")]
pub struct Post_state;

#[derive(
    Debug,
    FromSqlRow,
    AsExpression,
    Deserialize,
    Serialize,
    GraphQLEnum,
    WundergraphValue,
    Clone,
    Copy,
)]
#[sql_type = "Post_state"]
pub enum PostState {
    Draft,
    Published,
    Deleted,
}

#[derive(Deserialize, Debug)]
enum PostColumn {
    Id,
    Title,
    Content,
    PublishedAt,
    Author,
}

#[derive(Deserialize, Debug)]
enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Deserialize, Debug)]
struct Query {
    order: Option<PostColumn>,
    order_direction: Option<OrderDirection>,
    id: Option<i32>,
    title: Option<String>,
    content: Option<String>,
    later_than: Option<DateTime<Utc>>,
    author: Option<i32>,
}

impl FromSql<Post_state, Pg> for PostState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match bytes {
            Some(b"Draft") => Ok(PostState::Draft),
            Some(b"Published") => Ok(PostState::Published),
            Some(b"Deleted") => Ok(PostState::Deleted),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToSql<Post_state, Pg> for PostState {
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, Pg>) -> serialize::Result {
        match self {
            PostState::Draft => out.write_all(b"Draft")?,
            PostState::Published => out.write_all(b"Published")?,
            PostState::Deleted => out.write_all(b"Deleted")?,
        }
        Ok(serialize::IsNull::No)
    }
}

fn new_post(req: HttpRequest, new_post: Json<NewPost>) -> Result<Json<Post>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;
    Ok(diesel::insert_into(posts::table)
        .values((new_post.0, posts::post_state.eq(PostState::Draft)))
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

    diesel::update(posts::table.find(id.into_inner()))
        .set(posts::post_state.eq(PostState::Deleted))
        .execute(&conn)?;
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

fn build_post_query(query: Query) -> diesel::dsl::IntoBoxed<'static, posts::table, Pg> {
    let mut post_query = posts::table.into_boxed();

    if let Some(id) = query.id {
        post_query = post_query.filter(posts::id.eq(id));
    }
    if let Some(title) = query.title {
        post_query = post_query.filter(posts::title.like(title));
    }
    if let Some(content) = query.content {
        post_query = post_query.filter(posts::content.like(content));
    }
    if let Some(later_than) = query.later_than {
        post_query = post_query.filter(posts::published_at.ge(later_than));
    }
    if let Some(author) = query.author {
        post_query = post_query.filter(posts::author.eq(author));
    }

    match (query.order, query.order_direction) {
        (Some(PostColumn::Id), Some(OrderDirection::Desc)) => {
            post_query = post_query.order_by(posts::id.desc());
        }
        (Some(PostColumn::Id), _) => {
            post_query = post_query.order_by(posts::id);
        }
        (Some(PostColumn::Title), Some(OrderDirection::Desc)) => {
            post_query = post_query.order_by(posts::title.desc());
        }
        (Some(PostColumn::Title), _) => {
            post_query = post_query.order_by(posts::title);
        }
        (Some(PostColumn::Content), Some(OrderDirection::Desc)) => {
            post_query = post_query.order_by(posts::content.desc());
        }
        (Some(PostColumn::Content), _) => {
            post_query = post_query.order_by(posts::content);
        }
        (Some(PostColumn::PublishedAt), Some(OrderDirection::Desc)) => {
            post_query = post_query.order_by(posts::published_at.desc());
        }
        (Some(PostColumn::PublishedAt), _) => {
            post_query = post_query.order_by(posts::published_at);
        }
        (Some(PostColumn::Author), Some(OrderDirection::Desc)) => {
            post_query = post_query.order_by(posts::author.desc());
        }
        (Some(PostColumn::Author), _) => {
            post_query = post_query.order_by(posts::author);
        }
        (None, _) => {}
    }

    post_query
}

fn get_posts_with_query(
    req: HttpRequest,
    web::Query(query): web::Query<Query>,
) -> Result<Json<Vec<Post>>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    let post_query = build_post_query(query);

    Ok(post_query.load(&conn).map(Json)?)
}

#[derive(Deserialize)]
struct PageSize {
    page_size: Option<u32>,
    #[serde(flatten)]
    query: Query,
}

#[derive(Serialize)]
struct PostPage {
    page_number: u32,
    posts: Vec<Post>,
    total_pages: u32,
}

fn paginated_posts(
    req: HttpRequest,
    page: web::Path<u32>,
    web::Query(query): web::Query<PageSize>,
) -> Result<Json<PostPage>, Error> {
    let conn = req
        .app_data::<AppState>()
        .expect("AppData set")
        .pool
        .get()?;

    let page = page.into_inner();

    let post_query = build_post_query(query.query);

    Ok(post_query
        .paginate(page as i64)
        .per_page(query.page_size.map(|c| c as _).unwrap_or(DEFAULT_PER_PAGE))
        .load_and_count_pages(&conn)
        .map(|(posts, total_pages)| PostPage {
            posts,
            total_pages: total_pages as u32,
            page_number: page,
        })
        .map(Json)?)
}
