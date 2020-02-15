require 'net/https'
require './parse-dotinfo-page'

def write_page(page)
  if page == 1
    uri = URI("https://community.tulpa.info/thread-game-last-one-to-post-wins")
  else
    uri = URI("https://community.tulpa.info/thread-game-last-one-to-post-wins?page=#{page}")
  end

  http = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = true
  http.verify_mode = OpenSSL::SSL::VERIFY_NONE #TODO
  resp = http.get(uri.request_uri, {"User-Agent" => "Mozilla/5.0 (Operating system info here) Ruby/#{RUBY_VERSION} JeanLucTulpaInfoScraperBot/0.2"})
  pp resp if $DEBUG
  File.write("pages/#{page}.html",resp.body)
  puts page
  sleep(0.5)
end
$pagefiles = []
def bla
  $pagefiles = Dir['pages/*.html'].sort_by{|fn| (File.basename(fn).to_i)}
end
bla
puts $pagefiles.last
write_page(File.basename($pagefiles.last).to_i)

while (a = parse_dotinfo_page(File.read($pagefiles.last)))[:current_page] != 1 && a[:posts].length == 10
  write_page(File.basename($pagefiles.last).to_i + 1)
  bla
end

require './proper-db'
require './step2'
