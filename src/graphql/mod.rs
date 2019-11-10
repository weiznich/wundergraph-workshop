use crate::model::comments::NewComment;
use crate::model::posts::PostState;
use crate::model::users::NewUser;
use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::prelude::*;
use juniper::{ExecutionResult, Executor, GraphQLInputObject, Selection, Value};
use wundergraph::prelude::*;
use wundergraph::query_builder::mutations::{HandleBatchInsert, HandleInsert, HandleUpdate};
use wundergraph::query_builder::selection::LoadingHandler;
use wundergraph::scalar::WundergraphScalarValue;

mod post_at_version;

use self::post_at_version::*;

#[derive(WundergraphEntity, Identifiable, Debug, Clone)]
#[table_name = "users"]
pub struct User {
    id: i32,
    name: String,
    joined_at: DateTime<Utc>,
    posts: HasMany<Post, posts::author>,
    comments: HasMany<Comment, comments::author>,
}

#[derive(WundergraphEntity, Identifiable, Debug, Clone)]
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

#[derive(WundergraphEntity, Identifiable, Debug, Clone)]
#[table_name = "comments"]
pub struct Comment {
    id: i32,
    comment: String,
    published_at: DateTime<Utc>,
    author: HasOne<i32, User>,
    post: HasOne<i32, Post>,
    #[column_name = "post"]
    posts_at_version: HasOne<i32, PostAtVersion>,
}

wundergraph::query_object! {
    Query {
        User,
        Post,
        Comment,
        PostAtVersion(version: Option<i32>),
    }
}

#[derive(GraphQLInputObject, Identifiable, AsChangeset)]
#[table_name = "users"]
pub struct UserChangeset {
    id: i32,
    name: String,
}

#[derive(GraphQLInputObject, Identifiable)]
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
                    posts::version_start.eq(0),
                    posts::version_end.eq(Option::<i32>::None),
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
                        posts::version_start.eq(0),
                        posts::version_end.eq(Option::<i32>::None),
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

impl HandleUpdate<Post, PostChangeset, Pg, PgConnection> for posts::table {
    fn handle_update(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<PgConnection, WundergraphScalarValue>,
        update: &PostChangeset,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| {
            let current_version = posts::table
                .select(diesel::dsl::max(posts::version_start))
                .filter(posts::id.eq(update.id))
                .get_result::<Option<i32>>(conn)?
                .unwrap_or(0);

            diesel::update(
                posts::table.filter(
                    posts::id
                        .eq(update.id)
                        .and(posts::version_start.eq(current_version)),
                ),
            )
            .set(posts::version_end.eq(Some(current_version + 1)))
            .execute(conn)?;

            let inserted = diesel::insert_into(posts::table)
                .values((
                    posts::id.eq(update.id),
                    posts::title.eq(&update.title),
                    posts::content.eq(&update.content),
                    posts::author.eq(update.author),
                    posts::post_state.eq(update.post_state),
                    posts::version_start.eq(current_version + 1),
                    posts::version_end.eq(Option::<i32>::None),
                ))
                .returning(posts::id)
                .get_result::<i32>(conn)?;

            let look_ahead = executor.look_ahead();

            let query = <Post as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
                .filter(posts::id.eq(inserted));
            let items = Post::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
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
