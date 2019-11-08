require 'nokogiri'
require 'uri'
require 'time'
require 'pp'

class Object
  def try
    yield self
  end
end

class NilClass
  def try
  end
end

def parse_dotinfo_page(content, scraped_at = Time.now)
  noko = Nokogiri::HTML(content)
  res = {}
  res[:current_page] = noko.at_css(".pagination span.pagination_current").text.strip.to_i
  res[:is_last_page] = noko.at_css(".pagination .pagination_next").nil?
  res[:last_page] = res[:is_last_page] ? res[:current_page] : noko.at_css(".pagination .pagination_last").try{|n| n.text.strip.to_i}
  res[:uses_relative_timestamps] = false
  last_el = noko.at_css(".pagination_last")
  if last_el.nil?
    res[:total_pages] = res[:current_page]
  else
    res[:total_pages] = last_el.text.strip.to_i
  end
  posts = []
  noko.css("table[id^=post_]").each do |post|
    #old post format
    parsed = {}
    parsed[:id] = post.attributes["id"].value.match(/post_(\d+)/)[1].to_i
    parsed[:index] = post.at_css('.float_right:nth-of-type(1) > strong > a').text.gsub(/[^0-9]/,'').to_i
    parsed[:user] = {}
    find_button = post.at_css('.postbit_find')
    if find_button.nil?
      parsed[:user][:anonymous] = true
      parsed[:user][:name] = "Anonymous"
    else
      parsed[:user][:anonymous] = false
      parsed[:user][:id]     = Hash[URI.decode_www_form(URI(find_button.attributes["href"].value).query)]["uid"].to_i
      parsed[:user][:name]   = post.at_css("td[class^=trow]>strong>span.largetext").text.strip
      parsed[:user][:online] = post.at_css("td[class^=trow] img.buddy_status").attributes["alt"].value == "Online"
      parsed[:user][:tagline]= post.at_css("td[class^=trow]>span.smalltext").children.first.text.strip
      group_el = post.at_css("td[class^=trow]>span.smalltext>img")
      if !group_el.nil?
        parsed[:user][:group]  = group_el.attributes["alt"].value
        parsed[:user][:group_img] = group_el.attributes["src"].value
      end
    end
    post_time = post.at_css("tr:nth-of-type(2)>[class^=trow]:nth-of-type(1)>.smalltext").text
    parsed.merge! parse_dotinfo_post_time(post_time, scraped_at)
    res[:uses_relative_timestamps] ||= parsed[:uses_relative_timestamps]
    body = post.at_css(".post_body")
    parsed[:post_body] = body.children.find_all{|e| e.class != Nokogiri::XML::Comment}.map(&:to_html).join
    parsed[:post_body_text] = body.text
    posts << parsed
  end
  noko.css("#posts>a[id^=pid]").each do |a|
    #10.times{puts "================="}
    #pp a
    post = a.next
    while post.text? || post.comment?
      post = post.next
    end

    #pp post
    
    parsed = {}
    parsed[:id] = a.attributes["id"].value.match(/pid(\d+)/)[1].to_i
    parsed[:index] = post.at_css('.post_content>.post_head strong a').text.gsub(/[^0-9]/,'').to_i
    parsed[:user] = {}

    find_button = post.at_css('.postbit_find')
    if find_button.nil?
      parsed[:user][:anonymous] = true
      parsed[:user][:name] = "Anonymous"
    else
      parsed[:user][:anonymous] = false
      parsed[:user][:id] = Hash[URI.decode_www_form(URI(find_button.attributes["href"].value).query)]["uid"].to_i
      parsed[:user][:name] = post.at_css(".author_information strong a").text.strip
      parsed[:user][:online] = post.at_css(".author_information img.buddy_status").attributes["alt"].value.strip == "Online"
      parsed[:user][:tagline] = post.at_css(".author_information .smalltext").text.strip
      group_el = post.at_css(".author_information .smalltext>img")
      if !group_el.nil?
        parsed[:user][:group] = group_el.attributes["alt"].value
        parsed[:user][:group_img] = group_el.attributes["src"].value
      else
        parsed[:user][:group] = parsed[:user][:group_img] = nil
      end
    end
    convenient_title = post.at_css(".post_content .post_date span[title]")
    help = nil
    if !convenient_title.nil? && /^(\d\d-\d\d-\d\d\d\d|\d\d\d\d-\d\d-\d\d)$/.match(convenient_title.attributes["title"].value).nil?
      post_time = convenient_title.attributes["title"].value
    else
      if !convenient_title.nil?
        help = convenient_title.attributes["title"].value
      end
      post_time = post.at_css(".post_content .post_date").text.strip
    end
    parsed.merge! parse_dotinfo_post_time(post_time, scraped_at, help)
    res[:uses_relative_timestamps] ||= parsed[:uses_relative_timestamps]
    body = post.at_css(".post_body")
    parsed[:post_body] = body.children.find_all{|e| !e.comment?}.map(&:to_html).join
    parsed[:post_body_text] = body.text.strip
    posts << parsed
  end
  res[:posts] = posts
  return res
end

def parse_dotinfo_post_time(text, scraped_at = Time.now, help = nil)
  scraped_at = scraped_at.utc
  res = {}
  res[:debug_original_post_time_text] = text
  res[:uses_relative_timestamps] = false
  case text
  when /(\d+) (hour|minute|second)s? ago/
    res[:uses_relative_timestamps] = true
    res[:posted_at] = scraped_at - ($1.to_i * case $2[0]
                                                 when "h"
                                                   60 * 60
                                                 when "m"
                                                   60
                                                 when "s"
                                                   1
                                                 end)
  when /(Today|Yesterday), (\d\d:\d\d(:\d\d)? (AM|PM)?)/i
    res[:uses_relative_timestamps] = true
    day = nil
    if help.nil?
      day = scraped_at
      if $1 == "Yesterday"
        day -= 60*60*24
      end
    else
      day = Time.strptime(help, "%m-%d-%Y")
      res[:uses_relative_timestamps] = false
    end
    res[:posted_at] = Time.parse $2 + " +0000", day
  when /(\d\d-\d\d-\d\d\d\d), (\d\d:\d\d(:\d\d)? (AM|PM)?)/i
    date = Time.strptime($1, "%m-%d-%Y")
    res[:posted_at] = Time.parse ($2 + " +0000"), date
  when /(\d\d\d\d-\d\d-\d\d), (\d\d:\d\d(:\d\d)? (AM|PM)?)/i
    date = Time.strptime($1, "%Y-%m-%d")
    res[:posted_at] = Time.parse ($2 + " +0000"), date
  else
    raise "Could not parse post time #{text.inspect} :("
  end
  return res
end
