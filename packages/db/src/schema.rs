// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id            -> Int4,
        email         -> Text,
        password_hash -> Text,
        created_at    -> Timestamptz,
    }
}

diesel::table! {
    videos (id) {
        id           -> Int4,
        user_id      -> Int4,
        filename     -> Text,
        original_url -> Text,
        status       -> Text,
        created_at   -> Timestamptz,
    }
}

diesel::table! {
    video_formats (id) {
        id         -> Int4,
        video_id   -> Int4,
        resolution -> Text,
        url        -> Text,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(videos -> users (user_id));
diesel::joinable!(video_formats -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    videos,
    video_formats,
);
