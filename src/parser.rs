use chrono::Utc;
use chrono::offset::TimeZone;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Child, Class, Descendant, Name, Predicate};
use serde::Deserialize;

use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct Page {
    pub page_count: NonZeroU32,
    pub page_current: NonZeroU32,
    #[allow(dead_code)]
    pub pages: Vec<NonZeroU32>,
    pub posts: Vec<Post>,
}

impl Page {
    pub fn is_last_page(&self) -> bool {
        self.page_current == self.page_count
    }
}

fn strip_begin_expecting<'a>(thing: &'a str, expect: &'a str) -> &'a str {
    if thing.starts_with(expect) {
        thing.split_at(expect.len()).1
    } else {
        panic!("Expecting {:?} at beginning of {:?}", expect, thing);
    }
}

impl From<Document> for Page {
    fn from(document: Document) -> Page {
        let pagination_el = document.find(Class("ipsPagination")).next().unwrap();
        let page_count = pagination_el.attr("data-pages").unwrap().parse().unwrap();
        let page_current = pagination_el
            .find(Class("ipsPagination_active"))
            .next()
            .unwrap()
            .text()
            .trim()
            .parse()
            .unwrap();

        let mut pages = Vec::new();
        for page_el in pagination_el.find(Class("ipsPagination_page")) {
            let a = page_el.first_child().unwrap().attr("data-page").unwrap();
            let b = a.parse().unwrap();
            pages.push(b);
        }

        let mut posts = Vec::new();
        for node in document.find(Name("article").and(Class("cPost"))) {
            posts.push(Post::from(node));
        }
        Page {
            page_count,
            page_current,
            pages,
            posts,
        }
    }
}

#[derive(Debug, Clone)]
pub enum User {
    Anonymous,
    Known { id: u64, name: String },
}

#[derive(Debug, Clone)]
pub struct Post {
    #[allow(dead_code)]
    pub id: u32,
    pub num: Option<u32>,
    pub user: User,
    pub posted_at: chrono::DateTime<Utc>,
}

impl From<Node<'_>> for Post {
    fn from(n: Node<'_>) -> Post {
        // eprintln!("-v- post -v-");
        // eprintln!("{}", n.html());
        // eprintln!("-^- post -^-");
        let name_el = n
            .find(Descendant(
                Child(
                    (Name("aside")).and(Class("cAuthorPane")),
                    (Name("h3")).and(Class("cAuthorPane_author")),
                ),
                Name("strong"),
            ))
            .next()
            .unwrap();

        let id = strip_begin_expecting(n.attr("id").unwrap(), "elComment_")
            .parse()
            .unwrap();

        let num = if let Some(postnumber_span) = n.find(Class("ks_postNumber")).next() {
            // dbg!(&postnumber_span);
            let postnumber_unstripped = postnumber_span.text();
            let postnumber_str = postnumber_unstripped.trim();
            if postnumber_str.is_empty() {
                None
            } else {
                Some(strip_begin_expecting(postnumber_str, "#").parse().unwrap())
            }
        } else {
            let mut a = n.find(Class("ipsComment_tools"));
            let b = a.next().unwrap();
            let c = b.find(Name("li")).last();
            let d = c.unwrap();
            let e = d.text();
            let f = e.trim();
            if f == "Share" {
                None
            } else {
                Some(strip_begin_expecting(f, "#").parse().unwrap())
            }
        };

        let user;
        if let Some(a) = name_el.find(Name("a")).next() {
            let id: u64 = strip_begin_expecting(
                a.attr("href").unwrap(),
                "https://community.tulpa.info/profile/",
            )
            .split('-')
            .next()
            .unwrap()
            .parse()
            .unwrap();
            let name = name_el.text().trim().to_owned();
            user = User::Known { id, name };
        } else {
            user = User::Anonymous;
        }

        #[derive(Debug, Deserialize)]
        struct QuoteData {
            pub timestamp: i64,
        }

        let comment_wrap_el = n.find(Attr("data-quotedata", ())).next().unwrap();
        let qd: QuoteData =
            serde_json::from_str(comment_wrap_el.attr("data-quotedata").unwrap()).unwrap();
        let posted_at = Utc.timestamp_opt(qd.timestamp, 0).single().unwrap();

        Post {
            id,
            num,
            user,
            posted_at,
        }
    }
}
