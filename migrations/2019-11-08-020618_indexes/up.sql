CREATE INDEX standings_sets_a ON standings_sets (ty, finished_at);
CREATE INDEX standings_a ON standings (set_rowid, is_anon, accrued_time);
CREATE INDEX pages_a ON pages (page_num,valid,valid_html);
