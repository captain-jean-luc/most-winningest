// @generated automatically by Diesel CLI.

diesel::table! {
    pages (rowid) {
        rowid -> Int4,
        page_num -> Int4,
        body -> Text,
        created_at -> Timestamptz,
        is_last_page -> Bool,
        valid -> Bool,
        valid_html -> Bool,
    }
}

diesel::table! {
    posts (rowid) {
        rowid -> Int4,
        pages_rowid -> Nullable<Int4>,
        post_num -> Int4,
        username -> Text,
        userid -> Nullable<Int4>,
        posted_at -> Timestamptz,
        linked_accounts -> Array<Nullable<Text>>,
        master_account -> Nullable<Text>,
        injected -> Bool,
    }
}

diesel::table! {
    standings (rowid) {
        rowid -> Int4,
        set_rowid -> Int4,
        name -> Text,
        accrued_time -> Int4,
        post_count -> Int4,
        is_anon -> Bool,
    }
}

diesel::table! {
    standings_sets (rowid) {
        rowid -> Int4,
        ty -> Text,
        finished_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(posts -> pages (pages_rowid));
diesel::joinable!(standings -> standings_sets (set_rowid));

diesel::allow_tables_to_appear_in_same_query!(pages, posts, standings, standings_sets,);
