table! {
    use diesel::sql_types::*;
    use crate::model::posts::Post_state;

    comments (id) {
        id -> Int4,
        comment -> Nullable<Text>,
        published_at -> Timestamptz,
        author -> Int4,
        post -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::posts::Post_state;

    posts (id) {
        id -> Int4,
        title -> Text,
        content -> Nullable<Text>,
        published_at -> Timestamptz,
        author -> Int4,
        post_state -> Post_state,
        version_start -> Int4,
        version_end -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::posts::Post_state;

    users (id) {
        id -> Int4,
        name -> Text,
        joined_at -> Timestamptz,
    }
}

joinable!(comments -> users (author));
joinable!(posts -> users (author));

allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
