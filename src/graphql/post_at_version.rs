use crate::from_sql_function;
use crate::graphql::{Comment, User};
use crate::model::posts::PostState;
use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::connection::Connection;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_dsl::methods;
use diesel::Identifiable;
use juniper::{LookAheadArgument, LookAheadMethods, LookAheadSelection};
use wundergraph::error::Result;
use wundergraph::graphql_type::{GraphqlWrapper, WundergraphGraphqlMapper};
use wundergraph::juniper_ext::FromLookAheadValue;
use wundergraph::query_builder::selection::fields::WundergraphBelongsTo;
use wundergraph::query_builder::selection::filter::{
    BuildFilter, BuildFilterHelper, FilterWrapper,
};
use wundergraph::query_builder::selection::BoxedQuery;
use wundergraph::query_builder::selection::LoadingHandler;
use wundergraph::query_builder::types::{HasMany, HasOne};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphContext;

from_sql_function! {
    posts_at_version(version: Nullable<Integer>) {
        id -> Int4,
        title -> Text,
        content -> Nullable<Text>,
        published_at -> Timestamptz,
        author -> Int4,
        post_state -> crate::model::posts::Post_state,
    }
}

#[derive(Clone, Debug, BuildFilterHelper, WundergraphBelongsTo)]
#[table_name = "posts_at_version"]
pub struct PostAtVersion {
    id: i32,
    title: String,
    content: Option<String>,
    published_at: DateTime<Utc>,
    author: HasOne<i32, User>,
    comments: HasMany<Comment, comments::post>,
    post_state: PostState,
}

impl HasTable for PostAtVersion {
    type Table = posts_at_version::posts_at_version;

    fn table() -> Self::Table {
        unimplemented!()
    }
}

impl<'a> Identifiable for &'a PostAtVersion {
    type Id = &'a i32;

    fn id(self) -> Self::Id {
        &self.id
    }
}

impl<Ctx> LoadingHandler<Pg, Ctx> for PostAtVersion
where
    Ctx: WundergraphContext + 'static,
    Ctx::Connection: Connection<Backend = Pg>,
{
    type Columns = (
        posts_at_version::id,
        posts_at_version::title,
        posts_at_version::content,
        posts_at_version::published_at,
        posts_at_version::author,
        posts_at_version::post_state,
    );
    type FieldList = (
        i32,
        String,
        Option<String>,
        DateTime<Utc>,
        HasOne<i32, User>,
        HasMany<Comment, comments::post>,
        PostState,
    );
    type PrimaryKeyIndex = wundergraph::helper::TupleIndex0;
    type Filter = FilterWrapper<Self, Pg, Ctx>;
    const FIELD_NAMES: &'static [&'static str] = &[
        "id",
        "title",
        "content",
        "published_at",
        "author",
        "comments",
        "post_state",
    ];
    const TYPE_NAME: &'static str = "PostsAtVersion";

    fn build_query<'a>(
        _global_args: &[LookAheadArgument<WundergraphScalarValue>],
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, Pg, Ctx>>
    where
        Self::Table: methods::BoxedDsl<
                'a,
                Pg,
                Output = BoxedSelectStatement<
                    'a,
                    diesel::dsl::SqlTypeOf<<Self::Table as Table>::AllColumns>,
                    Self::Table,
                    Pg,
                >,
            > + 'static,
        <Self::Filter as BuildFilter<Pg>>::Ret: AppearsOnTable<Self::Table>,
    {
        let version: Option<i32> = select
            .argument("version")
            .and_then(|v| FromLookAheadValue::from_look_ahead(&v.value()));
        let mut query = posts_at_version(version)
            .into_boxed()
            .select(<Self as LoadingHandler<Pg, Ctx>>::get_select(select)?);

        query = <Self as LoadingHandler<Pg, Ctx>>::apply_filter(query, select)?;
        query = <Self as LoadingHandler<Pg, Ctx>>::apply_limit(query, select)?;
        query = <Self as LoadingHandler<Pg, Ctx>>::apply_offset(query, select)?;
        query = <Self as LoadingHandler<Pg, Ctx>>::apply_order(query, select)?;

        Ok(query)
    }
}

impl<Ctx> WundergraphGraphqlMapper<Pg, Ctx> for PostAtVersion
where
    Ctx: WundergraphContext + 'static,
    Ctx::Connection: Connection<Backend = diesel::pg::Pg>,
{
    type GraphQLType = GraphqlWrapper<PostAtVersion, Pg, Ctx>;
}
