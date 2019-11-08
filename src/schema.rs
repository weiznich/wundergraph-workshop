table! {
    comments (id) {
        id -> Int4,
        comment -> Nullable<Text>,
        published_at -> Timestamptz,
        author -> Int4,
        post -> Int4,
    }
}

table! {
    posts (id) {
        id -> Int4,
        title -> Text,
        content -> Nullable<Text>,
        published_at -> Timestamptz,
        author -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Text,
        joined_at -> Timestamptz,
    }
}

joinable!(comments -> posts (post));
joinable!(comments -> users (author));
joinable!(posts -> users (author));

allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
