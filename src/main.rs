use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;

mod generate;
mod manual_schema;
mod parser;
mod schema;

use manual_schema::valid_posts;
use schema::{pages, posts, standings, standings_sets};

type DT = chrono::DateTime<Utc>;

#[derive(Debug, Insertable)]
#[diesel(table_name = pages)]
struct PageIns<'a> {
    page_num: i32,
    body: &'a str,
    created_at: DT,
    is_last_page: bool,
    valid: bool,
    valid_html: bool,
}

#[derive(Debug, Queryable)]
struct Page {
    rowid: i32,
    page_num: i32,
}

impl Page {
    fn cols() -> (pages::rowid, pages::page_num) {
        (pages::rowid, pages::page_num)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
struct PostIns<'a> {
    pages_rowid: i32,
    post_num: i32,
    username: &'a str,
    userid: Option<i32>,
    posted_at: DT,
    linked_accounts: Vec<String>,
    master_account: Option<String>,
}

#[derive(Debug, Queryable)]
struct Post {
    username: String,
    userid: Option<i32>,
    posted_at: DT,
    master: Option<String>,
}

impl Post {
    fn view_cols() -> (
        valid_posts::username,
        valid_posts::userid,
        valid_posts::posted_at,
        valid_posts::master_account,
    ) {
        (
            valid_posts::username,
            valid_posts::userid,
            valid_posts::posted_at,
            valid_posts::master_account,
        )
    }
}

const KNOWN_SYSTEMS: &[(&str, &[&str])] = &[
    ("jean-luc", &["Snow", "HenHenry"]),
    ("Luminesce", &["Reisen", "Tewi", "Flandre", "Lucilyn"]),
    ("Breloomancer", &["Thelmign"]),
    ("TB", &["Byakko", "Rena Bonnie"]),
    (
        "BearBaeBeau",
        &["Autumn Ren", "Ashley", "Joy", "Misha", "Gweneth"],
    ),
    (
        "Felight",
        &[
            "Indigo Blue",
            "Apollo Fire",
            "Dynamo Lux",
            "Gelato Sweet",
            "Piano Soul",
            "Radio Hiss",
        ],
    ),
    (
        "Ranger",
        &[
            "GrayTheCat",
            "Evergreen Shadow",
            "Jared The Fabulous",
            "Jerry The Fabulous",
            "Chrome Shadow",
            "Adriel Shadow",
            "Fhern Shadow",
            "Ian Shadow",
            "Moltosha Shadow",
            "Exabier Shadow",
            "System Corporation",
        ],
    ),
    (
        "TurboSimmie",
        &["Chloe - September13", "September13", "September 13"],
    ),
    ("Yakumo", &["Ido"]),
    ("Linkzelda", &["| Eva |", "| Ada |"]),
    ("Ice909", &["Janey_is_Better_Than_Pleeb"]),
    ("Shaula", &["Nightfall"]),
    ("Seagull", &["Reina Akabane"]),
    ("Lilith_", &["Gloomynoon", "Myo"]),
];

fn make_user_to_system() -> HashMap<&'static str, &'static str> {
    let mut res = HashMap::new();
    for (system_name, members) in KNOWN_SYSTEMS {
        for member_name in *members {
            res.insert(*member_name, *system_name);
        }
    }
    res
}

async fn page_into_db(
    cli: &reqwest::Client,
    conn: &mut diesel::PgConnection,
    page_num: i32,
) -> parser::Page {
    eprintln!("grabbing {}", page_num);
    let url = if page_num == 1 {
        "https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/".to_owned()
    } else {
        format!(
            "https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/page/{}/",
            page_num
        )
    };
    let content = cli.get(&url).send().await.unwrap().text().await.unwrap();
    dbg!(content.as_str());
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
    conn.transaction(|conn| {
        diesel::update(pages::table)
            .filter(pages::dsl::page_num.eq(ins.page_num))
            .set((
                pages::dsl::valid.eq(false),
                pages::dsl::valid_html.eq(false),
            ))
            .execute(conn)?;
        let page: Page = diesel::insert_into(pages::table)
            .values(&ins)
            .returning(Page::cols())
            .get_result(conn)?;
        for post in &pageinfo.posts {
            let username;
            let userid;
            match &post.user {
                parser::User::Anonymous => {
                    username = "Anonymous";
                    userid = None;
                }
                parser::User::Known { id, name } => {
                    username = name;
                    userid = Some(id);
                }
            }
            let pn: i32 = post.num.map(|a| a.try_into().unwrap()).unwrap_or(0i32);
            let ins = PostIns {
                pages_rowid: page.rowid,
                post_num: pn,
                username,
                userid: userid.map(|v| (*v).try_into().unwrap()),
                posted_at: post.posted_at,
                linked_accounts: vec![],
                master_account: None as Option<String>,
            };
            diesel::insert_into(posts::table)
                .values(&ins)
                .execute(conn)?;
        }
        maybe_page = Some(page);
        let res: Result<(), Box<dyn std::error::Error>> = Ok(());
        res
    })
    .unwrap();
    //maybe_page.unwrap()
    pageinfo
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dotenv_res = dotenvy::dotenv();
    if let Err(e) = dotenv_res {
        eprintln!("WARN: Failed to load .env: {:?}", e);
    }
    let pg_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let skip_download = std::env::var("SKIP_DOWNLOAD").unwrap_or_default() == "1";
    let mut conn = diesel::PgConnection::establish(&pg_url).unwrap();

    if !skip_download {
        let cli = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .gzip(true)
            .referer(false)
            .user_agent("LOTPW stats.jean-luc.org scraper bot by jean-luc")
            .timeout(core::time::Duration::from_secs(30))
            .build()?;

        use schema::pages::dsl as p_dsl;
        let mut last_page = || {
            let x: Option<Page> = p_dsl::pages
                .select(Page::cols())
                .filter(p_dsl::valid)
                .order(p_dsl::page_num.desc())
                .limit(1)
                .get_result(&mut conn)
                .optional()
                .unwrap();
            x
        };
        let mut next_page = last_page().map(|p| p.page_num).unwrap_or(1);
        loop {
            let new_page = page_into_db(&cli, &mut conn, next_page).await;
            if new_page.is_last_page() {
                break;
            }
            next_page += 1;
        }

        let pages: i64 = p_dsl::pages
            .filter(p_dsl::valid)
            .count()
            .get_result(&mut conn)
            .unwrap();
        eprintln!("We should have all those pages now. {} in total", pages);
    }

    use valid_posts::dsl as o_dsl;
    let posts: Vec<Post> = o_dsl::valid_posts
        .select(Post::view_cols())
        .order(o_dsl::posted_at.desc())
        .get_results(&mut conn)
        .unwrap();
    let set_rowid: i32 = diesel::insert_into(standings_sets::table)
        .values(standings_sets::dsl::ty.eq("Individual"))
        .returning(schema::standings_sets::dsl::rowid)
        .get_result(&mut conn)
        .unwrap();
    #[derive(Debug, Clone, Insertable)]
    #[diesel(table_name = standings)]
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
        let &mut (ref mut standing, _) = standings.entry(userid).or_insert((
            Standing {
                set_rowid,
                name: post.username.clone(),
                accrued_time: 0,
                post_count: 0,
                is_anon: post.userid.is_none(),
            },
            post.master.clone(),
        ));
        name_to_id.entry(post.username.clone()).or_insert(userid);
        standing.name = post.username.clone();
        let x: i32 = (last_time - post.posted_at)
            .num_seconds()
            .try_into()
            .unwrap();
        standing.accrued_time += x;
        standing.post_count += 1;
        last_time = post.posted_at;
    }
    let x: Vec<_> = standings.values().map(|a| &a.0).collect();
    diesel::insert_into(schema::standings::table)
        .values(x)
        .execute(&mut conn)
        .unwrap();
    diesel::update(schema::standings_sets::table)
        .filter(standings_sets::dsl::rowid.eq(set_rowid))
        .set(standings_sets::dsl::finished_at.eq(Utc::now()))
        .execute(&mut conn)
        .unwrap();

    let set_rowid: i32 = diesel::insert_into(standings_sets::table)
        .values(standings_sets::dsl::ty.eq("System"))
        .returning(schema::standings_sets::dsl::rowid)
        .get_result(&mut conn)
        .unwrap();

    let user_to_system = make_user_to_system();
    let ids: Vec<_> = standings.keys().copied().collect();
    for id in ids {
        let master;
        let sub_standing;
        {
            let &mut (ref mut sub_standing_ref, ref master_ref) = standings.get_mut(&id).unwrap();
            sub_standing_ref.set_rowid = set_rowid;
            master = master_ref.as_ref().map(|s| s.to_owned()).or(user_to_system
                .get(sub_standing_ref.name.as_str())
                .map(|s| (*s).to_string()));
            sub_standing = sub_standing_ref.clone();
        }
        let Some(name) = master else {
            continue;
        };
        let Some(otherid) = name_to_id.get(&name) else {
            continue;
        };
        let &mut (ref mut master_standing, _) = standings.get_mut(otherid).unwrap();
        master_standing.accrued_time += sub_standing.accrued_time;
        master_standing.post_count += sub_standing.post_count;
        standings.remove(&id);
    }
    let x: Vec<_> = standings.values().map(|a| &a.0).collect();
    diesel::insert_into(standings::table)
        .values(x)
        .execute(&mut conn)
        .unwrap();
    diesel::update(standings_sets::table)
        .filter(standings_sets::dsl::rowid.eq(set_rowid))
        .set(standings_sets::dsl::finished_at.eq(Utc::now()))
        .execute(&mut conn)
        .unwrap();

    generate::generate();

    Ok(())
}
