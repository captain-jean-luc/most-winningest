
drop view valid_posts;
delete from posts where injected;
alter table posts alter column pages_rowid set not null;
alter table posts drop column injected;
create view valid_posts AS select posts.* from posts,pages where posts.pages_rowid = pages.rowid and pages.valid;
