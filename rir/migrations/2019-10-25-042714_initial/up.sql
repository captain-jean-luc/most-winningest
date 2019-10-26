-- Your SQL goes here
create table pages (
    rowid serial primary key,
    page_num integer not null,
    body text not null,
    created_at timestamptz not null,
    is_last_page boolean not null,
    valid boolean not null,
    valid_html boolean not null
);

create table posts (
    rowid serial primary key,
    pages_rowid integer not null references pages(rowid),
    post_num integer not null,
    username text not null,
    userid integer,
    posted_at timestamptz not null,
    linked_accounts text[] not null,
    master_account text --if null, that means this is the master account
);

create table standings_sets (
    rowid serial primary key,
    ty text not null CHECK(ty = 'Individual' or ty = 'System'),
    finished_at timestamptz
);

create table standings (
    rowid serial primary key,
    set_rowid integer not null references standings_sets(rowid),
    name text not null,
    accrued_time integer not null,
    post_count integer not null,
    is_anon boolean not null
);

create view valid_posts AS select posts.* from posts,pages where posts.pages_rowid = pages.rowid and pages.valid;