require 'net/https'
require 'pp' if $DEBUG

start = ARGV[0].to_i
stop  = ARGV[1].to_i

raise if start == 0 || stop == 0

(start..stop).each do |page|
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
