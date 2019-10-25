
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
  DB.execute("insert into pages (page_num, filename, needs_update) VALUES (?, ?, ?)", [pagenum, fn, res[:is_last_page] ? 't' : 'f'])
  page_rowid = DB.execute("select last_insert_rowid()")[0][0]
  res[:posts].each do |post|
    DB.execute("insert into posts (pages_rowid, user_forumid, username, is_anon, posted_at) VALUES (?, ?, ?, ?, ?)", [
                 page_rowid,
                 post[:user][:id],
                 post[:user][:name],
                 post[:user][:anonymous] ? 't' : 'f',
                 post[:posted_at].getutc.to_i
               ])
  end
end

pages = Dir["pages/*.html"] 

pb = ProgressBar.create(total: pages.length)

DB.transaction do
  pages.each do |page_fn|
    pagenum = File.basename(page_fn).to_i
    if DB.execute("select count(*) from pages where page_num = ?", pagenum)[0][0] == 0
      parse_and_insert(page_fn, pagenum)
    end
    pb.increment
  end
  puts "finishing transaction"
end

DB.execute "create index if not exists flag_blarg on posts(posted_at)"
