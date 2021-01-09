#![feature(proc_macro_hygiene, type_ascription)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate maud;
#[macro_use] extern crate phf;

use chrono::{DateTime, Utc};

mod schema {
    include!("../schema.rs");
}

use schema::{standings_sets, standings};

mod manual_schema {
    include!("../manual_schema.rs");
}

use diesel::prelude::*;
use diesel::r2d2;
use iron::prelude::*;
use iron::status;
use maud::{DOCTYPE,PreEscaped};

//darn people with long names
static NAME_REPLACEMENTS: phf::Map<&'static str, &'static str> = phf_map! {
    "theimaginationborders" => "tib",
    "TheCrawlingCreepypasta" => "TCC",
    "NateAndTheTulpaTrio" => "Nate",
    "Quetzal the furdragon" => "Quetzal",
    "psychologicalKitty-Cat" => "pK-C",
    "EmilyisMYworld1202" => "EiMw1202",
    "ShadowTheFluffhog" => "Shadow",
    "SoulslikeSpiderlegs" => "SlSl",
    "fountain_and_flame" => "f&f",
    "AvengedSevenfold" => "Avenged7x",
    "RazzleDazzleDorito" => "RDD",
    "JackTheRadiaution" => "Jack",
    "MariaTheFictionkin" => "Maria",
};

#[derive(Debug, Clone, Queryable)]
struct Standing {
    rowid: i32,
    set_rowid: i32,
    name: String,
    accrued_time: i32,
    post_count: i32,
    is_anon: bool,
}

type DBPool = r2d2::Pool<r2d2::ConnectionManager<diesel::pg::PgConnection>>;
// type ArcPool = Arc<DBPool>;

fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a/b, a%b)
}

fn display_time(accrued_time:i32) -> String {
    let (hours,minutes) = divmod(accrued_time / 60, 60);
    let (days, hours) = divmod(hours, 24);
    let (weeks, days) = divmod(days, 7);
    format!("{}w {}d {:02}h {:02}m", weeks, days, hours, minutes)
}

fn main() {
    dotenv::dotenv().unwrap();
    let pg_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable.");
    let manager = r2d2::ConnectionManager::new(pg_url.as_str());
    let pool:DBPool = r2d2::Pool::new(manager).unwrap();

    fn display_standing(s: &Standing, is_indiv: bool, rank: u32) -> maud::Markup {
        let maybe_replacement_name = NAME_REPLACEMENTS.get( s.name.as_str() );
        html! {
            tr.indiv[is_indiv].syste[!is_indiv] {
                @if s.is_anon {
                    td align="right" {}
                } @else {
                    td align="right" { (rank) "." }
                }
                td align="right" {
                    .name {
                        @if let Some(replacement_name) = maybe_replacement_name {
                            abbr title=(s.name) { (replacement_name) }
                        } @else {
                            (s.name)
                        }
                    }
                }
                td align="right" { 
                    .display_time { (display_time(s.accrued_time)) }
                }
                td align="right" { (s.post_count) }
            }
        }
    }

    let listen_spec = std::env::var("LISTEN").expect("Missing LISTEN environment variable, should be in the form address:port");
    let _server = Iron::new(move |req: &mut Request| {
        if req.method != iron::method::Method::Get || req.url.path() != vec![""] {
            return Ok(Response::with((status::NotFound, "Not found")));
        }
        let conn = pool.get().unwrap();
        let (indiv_set_rowid, indiv_upd):(i32,Option<DateTime<Utc>>) = 
            standings_sets::table
            .filter(standings_sets::dsl::ty.eq("Individual").and(standings_sets::dsl::finished_at.is_not_null()))
            .order(standings_sets::dsl::finished_at.desc())
            .select((standings_sets::dsl::rowid, standings_sets::dsl::finished_at))
            .get_result(&conn).unwrap();
        let (syste_set_rowid, syste_upd):(i32,Option<DateTime<Utc>>) = 
            standings_sets::table
            .filter(standings_sets::dsl::ty.eq("System"    ).and(standings_sets::dsl::finished_at.is_not_null()))
            .order(standings_sets::dsl::finished_at.desc())
            .select((standings_sets::dsl::rowid, standings_sets::dsl::finished_at))
            .get_result(&conn).unwrap();
        let indiv_standings:Vec<Standing> = standings::table
            .filter(standings::dsl::set_rowid.eq(indiv_set_rowid))
            .order((standings::dsl::is_anon, standings::dsl::accrued_time.desc()))
            .get_results(&conn).unwrap();
        let syste_standings:Vec<Standing> = standings::table
            .filter(standings::dsl::set_rowid.eq(syste_set_rowid))
            .order((standings::dsl::is_anon, standings::dsl::accrued_time.desc()))
            .get_results(&conn).unwrap();
        let last_updated = std::cmp::min(indiv_upd.unwrap(),syste_upd.unwrap());
        let markup = html! {
            (DOCTYPE)
            html {
                head {
                    (PreEscaped(r#"<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no"><link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" integrity="sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T" crossorigin="anonymous">"#))
                    style {
                        "
                        .name { display: inline-block; }
                        /*.indiv { display: none }
                        #show_indiv:checked ~ table .syste { display: none }
                        #show_indiv:checked ~ table .indiv { display: table-row }*/"
                    }
                    title { "LOTPW Stats" }
                }
                body {
                    .container {
                        h1 { "LOTPW Stats" }
                        p { 
                            r#"Any time that "Anonymous" has accrued is at the bottom. Updated every minute, hopefully. Last updated "#
                            ( last_updated.to_rfc3339_opts(chrono::SecondsFormat::Secs, true) )
                            "."
                        }
                        hr/
                        // p {
                        //     "2020-08-07: This has now been updated to support the new forum software, however the new forum does not show linked accounts, so all scores are now individual scores."
                        // }
                        // hr/
                        // input#show_indiv type="checkbox" checked? disabled?{}
                        // label for="show_indiv" {
                        //     ( maud::PreEscaped("&nbsp;") )
                        //     "Show individual scores"
                        // }
                        table.table.table-striped.table-bordered.table-hover.table-sm {
                            thead {
                                tr {
                                    th { "Rank" }
                                    th { "Name" }
                                    th style="min-width:8.3em" { "Time" }
                                    th { "Posts" }
                                }
                            }
                            tbody {
                                @if false {
                                    @let mut count = 1;
                                    @for standing in &syste_standings {
                                        ( display_standing(standing, false, count) )
                                        @if !standing.is_anon {
                                            ( { count += 1; ""} )
                                        }
                                    }
                                    @if syste_standings.len() % 2 == 1 {
                                        tr style="display:none" {}
                                    }
                                }
                                @let mut count = 1;
                                @for standing in &indiv_standings {
                                    ( display_standing(standing, true, count) )
                                    @if !standing.is_anon {
                                        ( { count += 1; ""} )
                                    }
                                }
                            }
                        }
                        small {
                            "Made by Jean-luc"
                        }
                    }
                }
            }
        };
        Ok(Response::with((status::Ok, markup))):IronResult<Response>
    }).http(&listen_spec).unwrap();
    println!("listening on {}", &listen_spec);
}
