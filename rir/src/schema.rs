table! {
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

table! {
    posts (rowid) {
        rowid -> Int4,
        pages_rowid -> Int4,
        post_num -> Int4,
        username -> Text,
        userid -> Nullable<Int4>,
        posted_at -> Timestamptz,
        linked_accounts -> Array<Text>,
        master_account -> Nullable<Text>,
    }
}

table! {
    standings (rowid) {
        rowid -> Int4,
        set_rowid -> Int4,
        name -> Text,
        accrued_time -> Int4,
        post_count -> Int4,
        is_anon -> Bool,
    }
}

table! {
    standings_sets (rowid) {
        rowid -> Int4,
        ty -> Text,
        finished_at -> Nullable<Timestamptz>,
    }
}

joinable!(posts -> pages (pages_rowid));
joinable!(standings -> standings_sets (set_rowid));

allow_tables_to_appear_in_same_query!(
    pages,
    posts,
    standings,
    standings_sets,
);
