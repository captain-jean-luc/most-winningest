require './parse-dotinfo-page'

printed_something = false
pagefiles = Dir['pages/*.html'].sort_by{|fn| -(File.basename(fn).to_i)}
pagefiles.each do |pagefn|
  STDERR.puts pagefn
  res = parse_dotinfo_page(File.read(pagefn))
  if res[:uses_relative_timestamps]
    puts pagefn
    printed_something = true
  else
    #break
  end
end

if !printed_something
  puts "No updates needed, most recent page is:"
  puts pagefiles.first
end
