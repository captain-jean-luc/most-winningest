require './parse-dotinfo-page'
require 'pp'

pp parse_dotinfo_page(File.read(ARGV.pop || "res.html"))
