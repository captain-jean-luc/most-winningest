require './parse-dotinfo-page'
require 'time'

start = Time.parse("2014-04-15T00:00:00Z")
current_count = 0
Dir.chdir('pages') do
  Dir['*.html'].sort_by(&:to_i).each do |page_fn|
    puts "reading #{page_fn}" if $DEBUG
    page_html = File.read(page_fn)
    scraped_at = File.mtime(page_fn)
    #puts page_html
    page = parse_dotinfo_page(page_html, scraped_at)
    page[:posts].each do |post|
      while post[:posted_at] >= (start + 60*60*24)
        puts "#{start.strftime("%F")},#{current_count}"
        current_count = 0
        start += 60*60*24
      end
      current_count += 1
    end
  end
end
puts "#{start.strftime("%F")},#{current_count}"
