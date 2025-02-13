diesel::table! {
    valid_posts (rowid) {
        rowid -> Int4,
        pages_rowid -> Nullable<Int4>,
        post_num -> Int4,
        username -> Text,
        userid -> Nullable<Int4>,
        posted_at -> Timestamptz,
        linked_accounts -> Array<Text>,
        master_account -> Nullable<Text>,
        injected -> Bool,
    }
}
