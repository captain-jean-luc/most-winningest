require './db'
require 'ruby-progressbar'
DB.transaction do
  last_id = nil
  last_time = nil
  rows = DB.execute("select rowid, posted_at, previous_post_rowid, time_gain_seconds from posts order by posted_at DESC")
  pb = ProgressBar.create(total: rows.length)
  rows.each do |row|
    rowid = row[0]
    posted_at = row[1]
    if !last_id.nil?
      if row[2].nil?
        DB.execute("update posts set previous_post_rowid = ?, time_gain_seconds = ? where rowid = ?", [
                     last_id,
                     last_time - posted_at,
                     rowid
                   ])
      end
    end
    last_id = rowid
    last_time = posted_at
    pb.increment
  end
end

puts "done step2"
