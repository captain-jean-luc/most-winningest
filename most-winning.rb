require './parse-dotinfo-page'
require 'time'
require 'pp'

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
        elsif !prev_post[:user][:anonymous]
          users[prev_post[:user][:name]] += delta
          if false && prev_post[:user][:name] == "jean-luc" && delta > ( 3600 * 100 )
            puts prev_post[:post_body_text].strip
            puts prev_post[:id]
            puts (delta / 3600.0)
            puts "-"*40
          end
        end
      end
      prev_post = post
    end
  end
end

users.to_a.sort_by{|name,time| time}.each_with_index do |(name, time), i|
  mm, ss = time.divmod(60)
  hh, mm =   mm.divmod(60)
  dd, hh =   hh.divmod(24)
  puts "%03d.%s:% 4dd %02dh %02dm" % [(users.size - i), name.rjust(25), dd, hh, mm]
end
