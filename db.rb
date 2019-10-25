require 'sqlite3'

DB = SQLite3::Database.new "cache.db"


DB.execute_batch <<SQL
CREATE TABLE IF NOT EXISTS pages (
  rowid integer primary key not null,
  page_num integer not null,
  filename text not null,
  needs_update text not null --bool
);

CREATE TABLE IF NOT EXISTS posts (
  rowid integer primary key not null,
  pages_rowid integer not null,
  previous_post_rowid integer,
  username text not null,
  user_forumid integer, --null if anon
  is_anon text not null,
  posted_at integer not null,
  time_gain_seconds integer
)
SQL
