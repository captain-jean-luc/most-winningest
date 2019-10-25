# coding: utf-8
require 'sinatra'
require 'haml'
require './db'

get '/' do
  users = Hash.new() #user id => [most recent name, length in seconds of winningness]
  DB.execute("select username, user_forumid, is_anon, time_gain_seconds, posted_at from posts order by posted_at").each do |row|
    username = row[0]
    id = row[1]
    is_anon = row[2] == 't'
    time = row[3]
    posted_at = Time.at(row[4])
    if !is_anon && !time.nil?
      curr_data = users[id] ||= ["",0,0]
      curr_data[2] += 1
      curr_data[1] += time
      curr_data[0] = username
      users[id] = curr_data
    elsif !is_anon
      curr_data = users[id] ||= ["",0,0]
      curr_data[2] += 1
      curr_data[1] += Time.now - posted_at
      curr_data[0] = username
      users[id] = curr_data
    end
  end

  @rows = users.to_a.sort_by{|_, (name,time,count)| -time}.each_with_index.map do |(id, (name, time, count)), i|
    name_num_spaces = [25 - name.size,0].max
    if name == "あんこ"
      name_num_spaces -= 2 #hack to align stuff, 2 spaces is specific to tulpa.info, it could be different in a terminal or even a different theme.
    end
    name_spaces = " " * name_num_spaces
    mm, ss = time.divmod(60)
    hh, mm =   mm.divmod(60)
    dd, hh =   hh.divmod(24)
    ww, dd =   dd.divmod( 7)
    #puts "%03d.%s: %2dw %dd %02dh %02dm" % [(users.size - i), name_spaces + name, ww, dd, hh, mm]
    [i+1, name, "%2dw %dd %02dh %02dm" % [ww, dd, hh, mm],count]
  end

  haml :index
end
