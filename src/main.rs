#![feature(type_ascription)]
#[macro_use] extern crate diesel;

use std::convert::TryInto;
use std::collections::HashMap;
use select::document::Document;
use select::predicate::{Predicate, Attr, Name, /*Class, Child, Element*/};
use diesel::prelude::*;
use chrono::Utc;

mod parser;
mod schema;
mod manual_schema;

use schema::{pages, posts, standings_sets, standings};
use manual_schema::{valid_posts};

type DT = chrono::DateTime<Utc>;

#[derive(Debug, Insertable)]
#[table_name="pages"]
struct PageIns<'a> {
    page_num:i32,
    body:&'a str,
    created_at:DT,
    is_last_page:bool,
    valid:bool,
    valid_html:bool,
}

#[derive(Debug, Queryable)]
struct Page{
    rowid:i32,
    page_num:i32,
    body:String,
    created_at:DT,
    is_last_page:bool,
    valid:bool,
    valid_html:bool,
}

#[derive(Debug, Insertable)]
#[table_name="posts"]
struct PostIns<'a> {
    pages_rowid:i32,
    post_num:i32,
    username:&'a str,
    userid:Option<i32>,
    posted_at:DT,
    linked_accounts: Vec<String>,
    master_account: Option<String>,
}

#[derive(Debug, Queryable)]
struct Post {
    rowid:i32,
    pages_rowid:i32,
    post_num:i32,
    username:String,
    userid:Option<i32>,
    posted_at:DT,
    linked_accounts: Vec<String>,
    master: Option<String>,
}

async fn page_into_db(cli: &reqwest::Client, conn: &diesel::PgConnection, page_num:i32) -> parser::Page {
    eprintln!("grabbing {}", page_num);
    let url;
    if page_num == 1 {
        url = "https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/".to_owned();
    } else {
        url = format!("https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/page/{}/", page_num);
    }
    let content = cli.get(&url).send().await.unwrap().text().await.unwrap();
    let pageinfo = parser::Page::from(select::document::Document::from(content.as_str()));
    let ins = PageIns {
        page_num: pageinfo.page_current.get().try_into().unwrap(),
        body: &content,
        created_at: Utc::now(),
        is_last_page: pageinfo.is_last_page(),
        valid: true,
        valid_html: true,
    };

    let mut maybe_page = None;
    conn.transaction(|| {
        diesel::update(pages::table).filter(pages::dsl::page_num.eq(ins.page_num)).set((
            pages::dsl::valid.eq(false),
            pages::dsl::valid_html.eq(false),
        )).execute(conn)?;
        let page:Page = diesel::insert_into(pages::table).values(&ins).get_result(conn)?;
        for post in &pageinfo.posts {
            let username;
            let userid;
            match &post.user {
                parser::User::Anonymous => {
                    username = "Anonymous";
                    userid = None;
                },
                parser::User::Known{id, name} => {
                    username = &name;
                    userid = Some(id);
                }
            }
            let pn:i32 = post.num.map(|a| a.try_into().unwrap()).unwrap_or(0i32);
            let ins = PostIns {
                pages_rowid: page.rowid,
                post_num: pn,
                username,
                userid: userid.map(|v| (*v).try_into().unwrap()),
                posted_at: post.posted_at,
                linked_accounts: vec![],
                master_account: None as Option<String>,
            };
            diesel::insert_into(posts::table).values(&ins).execute(conn)?;
        }
        maybe_page = Some(page);
        Ok(()):Result<(),Box<dyn std::error::Error>>
    }).unwrap();
    //maybe_page.unwrap()
    pageinfo
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();
    // let password = std::env::var("MW_PASSWORD").expect("Missing MW_PASSWORD");
    let pg_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let conn = diesel::PgConnection::establish(&pg_url).unwrap();
    
    let cli = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .gzip(true)
        .referer(false)
        .timeout(core::time::Duration::from_secs(30))
        .build()?;

    // let login_get_url = "https://community.tulpa.info/member.php?action=login";
    // let login_get_resp = cli.get(login_get_url).send().await.unwrap();
    // dbg!(login_get_resp.status());
    // let document = Document::from(login_get_resp.text().await.unwrap().as_str());
    // let login_form = document.find(Name("form").and(Attr("action","member.php"))).next().unwrap();
    // //get fields action, url, and my_post_key
    // let mut fields = login_form.find(Name("input").and(Attr("type","hidden")))
    //     .map(|node| (node.attr("name").unwrap_or(""), node.attr("value").unwrap_or("")))
    //     .collect::<Vec<_>>();
    // fields.push(("username","jean-luc-bot"));
    // fields.push(("password",&password));
    // fields.push(("remember","yes"));
    // fields.push(("submit","Login"));

    // let login_post = cli.post("https://community.tulpa.info/member.php").form(&fields).send().await?;
    // assert!(login_post.status().is_success());
    
    // //we don't actually care about the content, but we do want to wait for .info to finish sending the response
    // let _ = login_post.bytes().await;

    use schema::pages::dsl as p_dsl;
    let last_page = || {
        p_dsl::pages.filter(p_dsl::valid).order(p_dsl::page_num.desc()).limit(1).get_result(&conn).optional().unwrap():Option<Page>
    };
    let mut next_page = last_page().map(|p| p.page_num).unwrap_or(1);
    loop {
        let new_page = page_into_db(&cli, &conn, next_page).await;
        if new_page.is_last_page() { break }
        next_page += 1;
    }

    eprintln!("We should have all those pages now. {} in total", p_dsl::pages.filter(p_dsl::valid).count().get_result(&conn).unwrap():i64);

    use valid_posts::dsl as o_dsl;
    let posts:Vec<Post> = o_dsl::valid_posts
        .order(o_dsl::post_num.desc())
        .get_results(&conn)
        .unwrap();
    // let name_to_id = HashMap::<String, i32>::new()
    // for post in &posts {

    // }
    let set_rowid:i32 = diesel::insert_into(standings_sets::table).values(standings_sets::dsl::ty.eq("Individual")).returning(schema::standings_sets::dsl::rowid).get_result(&conn).unwrap();
    #[derive(Debug, Clone, Insertable)]
    #[table_name="standings"]
    struct Standing {
        set_rowid: i32,
        name: String,
        accrued_time: i32,
        post_count: i32,
        is_anon: bool,
    }
    let mut name_to_id = HashMap::<String, i32>::new();
    let mut standings = HashMap::<i32, (Standing, Option<String>)>::new();
    let mut last_time = Utc::now();
    for post in &posts {
        let userid = post.userid.unwrap_or(0);
        let (ref mut standing, _) = standings.entry(userid).or_insert((
            Standing{set_rowid, name: post.username.clone(), accrued_time: 0, post_count: 0, is_anon: post.userid.is_none()},
            post.master.clone(),
        ));
        name_to_id.entry(post.username.clone()).or_insert(userid);
        standing.name = post.username.clone();
        standing.accrued_time += (last_time - post.posted_at).num_seconds().try_into().unwrap():i32;
        standing.post_count += 1;
        last_time = post.posted_at;
    }
    diesel::insert_into(schema::standings::table).values(standings.values().map(|a| &a.0).collect():Vec<_>).execute(&conn).unwrap();
    diesel::update(schema::standings_sets::table).filter(standings_sets::dsl::rowid.eq(set_rowid)).set(standings_sets::dsl::finished_at.eq(Utc::now())).execute(&conn).unwrap();

    let set_rowid:i32 = diesel::insert_into(standings_sets::table).values(standings_sets::dsl::ty.eq("System")).returning(schema::standings_sets::dsl::rowid).get_result(&conn).unwrap();

    let manual_fixings:HashMap<&'static str, &'static str> = [
        ( "Snow", "jean-luc" ),
        ( "HenHenry", "jean-luc" ),
    ].iter().cloned().collect();
    let ids = standings.keys().map(|k| *k).collect():Vec<_>;
    for id in ids {
        let master;
        let sub_standing;
        {
            let (ref mut sub_standing_ref, ref master_ref) = standings.get_mut(&id).unwrap();
            // if &sub_standing_ref.name == "Snow" {
            //     dbg!(&sub_standing_ref, &master_ref);
            // }
            sub_standing_ref.set_rowid = set_rowid;
            master = master_ref.as_ref().map(|s| s.to_owned()).or(manual_fixings.get(sub_standing_ref.name.as_str()).map(|s| (*s).to_string()));
            sub_standing = sub_standing_ref.clone();
        }
        if let Some(name) = master {
            if let Some(otherid) = name_to_id.get(&name) {
                let (ref mut master_standing, _) = standings.get_mut(&otherid).unwrap();
                master_standing.accrued_time += sub_standing.accrued_time;
                master_standing.post_count += sub_standing.post_count;
                standings.remove(&id);
            }
        }
    }
    diesel::insert_into(standings::table).values(standings.values().map(|a| &a.0).collect():Vec<_>).execute(&conn).unwrap();
    diesel::update(standings_sets::table).filter(standings_sets::dsl::rowid.eq(set_rowid)).set(standings_sets::dsl::finished_at.eq(Utc::now())).execute(&conn).unwrap();

    Ok(())
}
