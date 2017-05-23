require './parse-dotinfo-page'

Dir['pages/*.html'].sort_by{|fn| -(fn.gsub(/[0-9]/,'').to_i)}.each do |pagefn|
  res = parse_dotinfo_page(File.read(pagefn))
  if res[:uses_relative_timestamps]
    puts pagefn
  else
    #break
  end
end
