require './parse-dotinfo-page'
require 'pp'

puts "test"
pp parse_dotinfo_page(File.read(ARGV.pop || "res.html"))
