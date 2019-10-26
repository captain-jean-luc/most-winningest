#![feature(proc_macro_hygiene, type_ascription)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate maud;

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
    let pg_url = std::env::var("DATABASE_URL").unwrap();
    let manager = r2d2::ConnectionManager::new(pg_url.as_str());
    let pool:DBPool = r2d2::Pool::new(manager).unwrap();

    fn display_standing(s: &Standing, is_indiv: bool, rank: u32) -> maud::Markup {
        html! {
            tr.indiv[is_indiv].syste[!is_indiv] {
                @if s.is_anon {
                    td align="right" {}
                } @else {
                    td align="right" { (rank) "." }
                }
                td align="right" { (s.name) }
                td align="right" { (display_time(s.accrued_time)) }
                td align="right" { (s.post_count) }
            }
        }
    }

    let listen_spec = std::env::var("LISTEN").unwrap();
    let _server = Iron::new(move |_: &mut Request| {
        let conn = pool.get().unwrap();
        let indiv_set_rowid:i32 = standings_sets::table.filter(standings_sets::dsl::ty.eq("Individual").and(standings_sets::dsl::finished_at.is_not_null())).order(standings_sets::dsl::finished_at.desc()).select(standings_sets::dsl::rowid).get_result(&conn).unwrap();
        let syste_set_rowid:i32 = standings_sets::table.filter(standings_sets::dsl::ty.eq("System"    ).and(standings_sets::dsl::finished_at.is_not_null())).order(standings_sets::dsl::finished_at.desc()).select(standings_sets::dsl::rowid).get_result(&conn).unwrap();
        let indiv_standings:Vec<Standing> = standings::table.filter(standings::dsl::set_rowid.eq(indiv_set_rowid)).order((standings::dsl::is_anon, standings::dsl::accrued_time.desc())).get_results(&conn).unwrap();
        let syste_standings:Vec<Standing> = standings::table.filter(standings::dsl::set_rowid.eq(syste_set_rowid)).order((standings::dsl::is_anon, standings::dsl::accrued_time.desc())).get_results(&conn).unwrap();
        let markup = html! {
            (DOCTYPE)
            html {
                head {
                    (PreEscaped(r#"<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no"><link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" integrity="sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T" crossorigin="anonymous">"#))
                    style {
                        ".indiv { display: none }
                        #show_indiv:checked ~ table .syste { display: none }
                        #show_indiv:checked ~ table .indiv { display: table-row }"
                    }
                    title { "LOTPW Stats" }
                }
                body {
                    .container {
                        h1 { "LOTPW Stats" }
                        p { r#"Any time that "Anonymous" has accrued is at the bottom. Updated every minute"# }
                        hr {}
                        input#show_indiv type="checkbox" {}
                        label for="show_indiv" {
                            ( maud::PreEscaped("&nbsp;") )
                            "Show individual scores"
                        }
                        table.table.table-striped.table-bordered.table-hover.table.sm {
                            thead {
                                tr {
                                    th { "Rank" }
                                    th { "Name" }
                                    th style="min-width:8em" { "Time" }
                                    th { "Posts" }
                                }
                            }
                            tbody {
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