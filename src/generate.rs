use chrono::{DateTime, Utc};

use crate::schema::{standings_sets, standings};

use diesel::prelude::*;
use diesel::r2d2;
use maud::{DOCTYPE,PreEscaped,html};
use phf::phf_map;

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
    name: String,
    accrued_time: i32,
    post_count: i32,
    is_anon: bool,
}

impl Standing {
    fn cols() -> (standings::name, standings::accrued_time, standings::post_count, standings::is_anon) {
        (standings::name, standings::accrued_time, standings::post_count, standings::is_anon)
    }
}

type DBPool = r2d2::Pool<r2d2::ConnectionManager<diesel::pg::PgConnection>>;

fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a/b, a%b)
}

fn display_time(accrued_time:i32) -> String {
    let (hours,minutes) = divmod(accrued_time / 60, 60);
    let (days, hours) = divmod(hours, 24);
    let (weeks, days) = divmod(days, 7);
    format!("{}w {}d {:02}h {:02}m", weeks, days, hours, minutes)
}

pub fn generate() {
    eprintln!("Generating...");
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

    let conn = pool.get().unwrap();
    let (indiv_set_rowid, indiv_upd):(i32,Option<DateTime<Utc>>) = 
        standings_sets::table
        .filter(standings_sets::dsl::ty.eq("Individual").and(standings_sets::dsl::finished_at.is_not_null()))
        .order(standings_sets::dsl::finished_at.desc())
        .select((standings_sets::dsl::rowid, standings_sets::dsl::finished_at))
        .get_result(&conn).unwrap();
    let indiv_standings:Vec<Standing> = standings::table
        .select(Standing::cols())
        .filter(standings::dsl::set_rowid.eq(indiv_set_rowid))
        .order((standings::dsl::is_anon, standings::dsl::accrued_time.desc()))
        .get_results(&conn).unwrap();
    let last_updated = indiv_upd.unwrap();
    let markup = html! {
        (DOCTYPE)
        html {
            head {
                (PreEscaped(r#"<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no"><link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" integrity="sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T" crossorigin="anonymous">"#))
                style {
                    ".name { display: inline-block; }"
                }
                title { "LOTPW Stats" }
            }
            body {
                .container {
                    h1 { "LOTPW Stats" }
                    p { 
                        r#"Any time that Anonymous/Guest posters have accrued is at the bottom. Updated every once in awhile, hopefully. Last updated "#
                        ( last_updated.to_rfc3339_opts(chrono::SecondsFormat::Secs, true) )
                        "."
                    }
                    p { "Has more than a few days passed since it last updated? That means something broke, please let me know." }
                    hr/
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
                            @let mut count = 1;
                            @for standing in &indiv_standings {
                                ( display_standing(standing, true, count) )
                                @if !standing.is_anon {
                                    ( { count += 1; ""} )
                                }
                            }
                        }
                    }
                    p {
                        "Note: Posts from the April 14th outage should be included. All the info was taken from "
                        a href="https://community.tulpa.info/topic/7356-game-last-one-to-post-wins/page/6595/?tab=comments#comment-296381" { "Pleeb's post" }
                        " and manually imported. No free 10 hours for Felicity."
                    }
                    small {
                        "Made by Jean-luc"
                    }
                }
            }
        }
    };

    let location = "generated/index.html";
    std::fs::create_dir_all("generated").unwrap();
    std::fs::write(location, markup.into_string()).unwrap();
    eprintln!("Wrote generated file to {}", location);
}
