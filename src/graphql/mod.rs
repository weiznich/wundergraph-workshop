use crate::model::comments::NewComment;
use crate::model::posts::PostState;
use crate::model::users::NewUser;
use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::prelude::*;
use juniper::{ExecutionResult, Executor, GraphQLInputObject, Selection, Value};
use wundergraph::prelude::*;
use wundergraph::query_builder::mutations::{HandleBatchInsert, HandleInsert};
use wundergraph::query_builder::selection::LoadingHandler;
use wundergraph::scalar::WundergraphScalarValue;

#[derive(WundergraphEntity, Identifiable, Debug)]
#[table_name = "users"]
pub struct User {
    id: i32,
    name: String,
    joined_at: DateTime<Utc>,
    posts: HasMany<Post, posts::author>,
    comments: HasMany<Comment, comments::author>,
}

#[derive(WundergraphEntity, Identifiable, Debug)]
#[table_name = "posts"]
pub struct Post {
    id: i32,
    title: String,
    content: Option<String>,
    published_at: DateTime<Utc>,
    author: HasOne<i32, User>,
    comments: HasMany<Comment, comments::post>,
    post_state: PostState,
}

#[derive(WundergraphEntity, Identifiable, Debug)]
#[table_name = "comments"]
pub struct Comment {
    id: i32,
    comment: String,
    published_at: DateTime<Utc>,
    author: HasOne<i32, User>,
    post: HasOne<i32, Post>,
}

wundergraph::query_object! {
    Query {
        User,
        Post,
        Comment,
    }
}

#[derive(GraphQLInputObject, Identifiable, AsChangeset)]
#[table_name = "users"]
pub struct UserChangeset {
    id: i32,
    name: String,
}

#[derive(GraphQLInputObject, Identifiable, AsChangeset)]
#[table_name = "posts"]
pub struct PostChangeset {
    id: i32,
    title: String,
    content: Option<String>,
    author: i32,
    post_state: PostState,
}

#[derive(GraphQLInputObject, Identifiable, AsChangeset)]
#[table_name = "comments"]
pub struct CommentChangeset {
    id: i32,
    comment: String,
    author: i32,
    post: i32,
}

#[derive(Debug, GraphQLInputObject)]
pub struct NewPost {
    title: String,
    content: Option<String>,
    author: i32,
}

impl HandleInsert<Post, NewPost, Pg, PgConnection> for posts::table {
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, PgConnection, WundergraphScalarValue>,
        insertable: NewPost,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| {
            let look_ahead = executor.look_ahead();
            let inserted = diesel::insert_into(posts::table)
                .values((
                    posts::title.eq(insertable.title),
                    posts::content.eq(insertable.content),
                    posts::author.eq(insertable.author),
                    posts::post_state.eq(PostState::Draft),
                ))
                .returning(posts::id)
                .get_result::<i32>(conn)?;

            let query = <Post as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
                .filter(posts::id.eq(inserted));
            let items = Post::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

impl HandleBatchInsert<Post, NewPost, Pg, PgConnection> for posts::table {
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, PgConnection, WundergraphScalarValue>,
        insertable: Vec<NewPost>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        let insert = insertable
            .into_iter()
            .map(
                |NewPost {
                     title,
                     content,
                     author,
                 }| {
                    (
                        posts::title.eq(title),
                        posts::content.eq(content),
                        posts::author.eq(author),
                        posts::post_state.eq(PostState::Draft),
                    )
                },
            )
            .collect::<Vec<_>>();
        conn.transaction(|| {
            let look_ahead = executor.look_ahead();
            let inserted = diesel::insert_into(posts::table)
                .values(insert)
                .returning(posts::id)
                .get_results::<i32>(conn)?;

            let query = <Post as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
                .filter(posts::id.eq_any(inserted));
            let items = Post::load(&look_ahead, selection, executor, query)?;
            Ok(Value::list(items))
        })
    }
}

wundergraph::mutation_object! {
    Mutation {
        User(insert = NewUser, update = UserChangeset, delete = true),
        Post(insert = NewPost, update = PostChangeset, delete = true),
        Comment(insert = NewComment, update = CommentChangeset, delete = true),
    }
}
