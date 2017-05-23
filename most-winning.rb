require './parse-dotinfo-page'
require 'time'

users = Hash.new(0) #name: length in seconds of winningness
prev_post = nil

Dir.chdir('pages') do
  Dir['*.html'].sort_by(&:to_i).each do |page_fn|
    puts "reading #{page_fn}" if $DEBUG
    page_html = File.read(page_fn)
    scraped_at = File.mtime(page_fn)
    #puts page_html
    page = parse_dotinfo_page(page_html, scraped_at)
    page[:posts].each do |post|
      unless prev_post.nil?
        delta = (post[:posted_at] - prev_post[:posted_at])
        if delta < 0
          STDERR.puts "whaaaa? Negative delta! #{page_fn} #{post[:index]} #{delta}"
          STDERR.pp prev_post
          STDERR.pp post
        else
          users[prev_post[:user][:name]] += delta
        end
      end
      prev_post = post
    end
  end
end

users.to_a.sort_by{|name,time| time}.each do |name, time|
  mm, ss = time.divmod(60)
  hh, mm =   mm.divmod(60)
  dd, hh =   hh.divmod(24)
  puts "%s: %02dd %02dh %02dm" % [name.rjust(25), dd, hh, mm]
end
