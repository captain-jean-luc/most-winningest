
require './parse-dotinfo-page'
require './db'
require 'ruby-progressbar'

def parse_and_insert(fn, pagenum)
  data = File.read(fn)
  res = parse_dotinfo_page(data)
  if res[:uses_relative_timestamps]
    STDERR.puts "PANIC and run into the streets"
    exit 1
  end
  if pagenum != 1 and res[:current_page] == 1
    return
  end
  DB.execute("select rowid from pages where page_num = ?", [pagenum]).each do |row|
    rowid = row[0]
    DB.execute("delete from posts where pages_rowid = ?", [rowid])
    DB.execute("delete from pages where rowid = ?", [rowid])
  end
  DB.execute("insert into pages (page_num, filename, needs_update, updated_at) VALUES (?, ?, ?, ?)", [pagenum, fn, res[:is_last_page] ? 't' : 'f', File.mtime(fn).to_i])
  page_rowid = DB.execute("select last_insert_rowid()")[0][0]
  res[:posts].each do |post|
    DB.execute("insert into posts (pages_rowid, user_forumid, username, is_anon, posted_at) VALUES (?, ?, ?, ?, ?)", [
                 page_rowid,
                 post[:user][:id],
                 post[:user][:name],
                 post[:user][:anonymous] ? 't' : 'f',
                 post[:posted_at].getutc.to_i
                 #File.mtime(fn)
               ])
  end
end

pages = Dir["pages/*.html"] 

pb = ProgressBar.create(total: pages.length)

DB.transaction do
  pages.each do |page_fn|
    pagenum = File.basename(page_fn).to_i
    rows = DB.execute("select rowid, updated_at from pages where page_num = ?", pagenum)
    if rows.length == 0 || rows[0][1].nil? || rows[0][1] < File.mtime(page_fn).to_i
      parse_and_insert(page_fn, pagenum)
    end
    pb.increment
  end
  puts "finishing transaction"
end

DB.execute "create index if not exists flag_blarg on posts(posted_at)"
