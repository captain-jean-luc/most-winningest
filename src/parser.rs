use chrono::Utc;
use select::node::Node;
use select::document::Document;
use select::predicate::{Predicate, Class, Name, Element, /*Attr, Child,*/};

use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct Page {
    pub page_count: NonZeroU32,
    pub page_current: NonZeroU32,
    pub page_first: Option<NonZeroU32>,
    pub page_last: Option<NonZeroU32>,
    pub pages: Vec<NonZeroU32>,
    pub posts: Vec<Post>,
}

impl Page {
    pub fn last_page(&self) -> NonZeroU32 { 
        let mut all_pages = self.pages.clone();
        all_pages.push(self.page_current);
        if let Some(n) = self.page_first { all_pages.push(n) }
        if let Some(n) = self.page_last  { all_pages.push(n) }
        all_pages.sort_unstable();

        *all_pages.last().unwrap()
    }

    pub fn is_last_page(&self) -> bool {
        self.last_page() == self.page_current
    }
}

fn strip_begin_expecting<'a>(thing: &'a str, expect:&'a str) -> &'a str {
    if thing.starts_with( expect ) {
        return thing.split_at(expect.len()).1
    }else{
        panic!("Expecting {:?} at beginning of {:?}", expect, thing);
    }
}

fn strip_end_expecting<'a>(thing: &'a str, expect:&'a str) -> &'a str {
    if thing.ends_with( expect ) {
        return thing.split_at(thing.len() - expect.len()).0
    }else{
        panic!("Expecting {:?} at beginning of {:?}", expect, thing);
    }
}

fn strip_both_expecting<'a>(expect_begin: &'a str, thing: &'a str, expect_end: &'a str) -> &'a str {
    strip_begin_expecting(strip_end_expecting(thing, expect_end), expect_begin)
}

impl From<Document> for Page {
    fn from(document: Document) -> Page {
        let pagination_el = document.find(Class("pagination")).next().unwrap();
        let pages_el = pagination_el.find(Class("pages")).next().unwrap();
        let pages_el_text = pages_el.text();
        let page_text = strip_both_expecting("Pages (",pages_el_text.trim(),"):");
        let page_count = page_text.parse().unwrap();
        let page_current = pagination_el.find(Class("pagination_current")).next().unwrap().text().trim().parse().unwrap();
        let page_first = pagination_el.find(Class("pagination_first")).next().map(|e| e.text().trim().parse().unwrap());
        let page_last = pagination_el.find(Class("pagination_last")).next().map(|e| e.text().trim().parse().unwrap());

        let mut pages = Vec::new();
        for page_el in pagination_el.find(Class("pagination_page")) {
            pages.push(page_el.text().trim().parse().unwrap())
        }

        let mut posts = Vec::new();
        for node in document.find(Class("post")){
            if let Some(val) = node.attr("id") {
                if val.starts_with("post_") {
                    posts.push(Post::from(node));
                }
            }
        }
        Page{
            page_count,
            page_current,
            page_first,
            page_last,
            pages,
            posts,
        }
    }
}

#[derive(Debug, Clone)]
pub enum User {
    Anonymous,
    Known{id: u64, name: String},
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id:u32,
    pub num:u32,
    pub user:User,
    pub post_content:String,
    pub posted_at:chrono::DateTime<Utc>,
    pub is_master_account:bool, //true for anon
    pub linked_accounts:Vec<(String,bool)>,
}

impl From<Node<'_>> for Post {
    fn from(n: Node<'_>) -> Post {
        let id = strip_begin_expecting(n.attr("id").unwrap(), "post_").parse().unwrap();
        let author_info_el = n.find(Class("post_author")).next().unwrap();

        let content_el = n.find(Class("post_content")).next().unwrap();

        let controls = n.find(Class("post_controls")).next().unwrap();

        let user;
        if let Some(el) = controls.find(Class("postbit_find")).next() {
            let id = strip_begin_expecting(el.attr("href").unwrap(), "search.php?action=finduser&uid=").parse().unwrap();
            let author_information_el = author_info_el.find(Class("author_information")).next().unwrap();
            let name = author_information_el.find(Name("span")).next().unwrap().text().trim().to_owned();
            user = User::Known{id, name};
        } else {
            user = User::Anonymous;
        }

        let post_content = content_el.find(Class("post_body")).next().unwrap().text().trim().to_owned();

        let post_head = content_el.find(Class("post_head")).next().unwrap();
        let num = post_head.find(Class("float_right")).next().unwrap().text().trim().replace('#', "").replace(',', "").parse().unwrap();
        let post_date = post_head.find(Class("post_date")).next().unwrap();
        let mut posted_at_string = String::new();
        for child in post_date.children() {
            if let Some(text) = child.as_text() {
                posted_at_string.push_str(text);
            }else if child.is(Class("post_edit")) {
                // do nothing
            }else if child.is(Name("span")) {
                posted_at_string.push_str(child.attr("title").unwrap());
            }
        }
        
        //dbg!(posted_at_string.trim());
        let ndt = chrono::NaiveDateTime::parse_from_str(posted_at_string.trim(), "%Y-%m-%d, %H:%M").unwrap();
        let posted_at = chrono::DateTime::<Utc>::from_utc(ndt, Utc);

        let mut is_master_account = true;
        let mut linked_accounts = Vec::new();
        if let User::Anonymous = user {
            is_master_account = true;
        }else{
            let author_stats_el = author_info_el.find(Class("author_statistics")).next().unwrap();
            for node in author_stats_el.children() {
                if let Some(val) = node.attr("id") {
                    if val.trim().starts_with("aj_postuser_") {
                        //the <n> Attached Accounts or Linked Accounts
                    } else if val.trim().starts_with("aj_postbit_") {
                        let head_text_uncut = node.find(Class("thead")).next().unwrap().text();
                        let head_text = head_text_uncut.trim();
                        if head_text == "Linked Accounts" {
                            is_master_account = false
                        } else if head_text == "Attached Accounts" {
                            is_master_account = true
                        }
                        let juicy_content = node.find(Class("trow1").or(Class("trow2"))).next().unwrap();
                        for li in juicy_content.find(Name("li")) {
                            let mut things = li.find(Element);
                            things.next();
                            let name_span = things.next().unwrap();
                            let is_master;
                            if let Some(val) = name_span.attr("title") {
                                assert_eq!(val.trim(), "Master Account");
                                is_master = true;
                            }else{ is_master = false }
                            linked_accounts.push((name_span.text().to_owned(), is_master));
                        }
                    }
                }
            }
            // if let User::Known{ref name, id: _} = user {
            //     if name == "Snow" {
            //         dbg!(author_stats_el.inner_html());
            //         std::process::exit(1);
            //     }
            // }
        }

        

        Post{
            id,
            num,
            user,
            post_content,
            posted_at,
            linked_accounts,
            is_master_account,
        }
    }
}
// impl From<Node<'_>> for Post {
//     fn from(n: Node<'_>) -> Post {
//         if n.name() == Some("table") {
//             let node_id = n.attr("id").unwrap();
//             if !node_id.starts_with("post_") {
//                 panic!();
//             }
//             let (_, post_id_str) = node_id.split_at(5);
//             let post_id:u64 = post_id_str.parse().unwrap();
//             let user:User;
//             if let Some(find_button) = n.find(Class("postbit_find")).next() {
//                 let name_span = n.find(
//                     Child(
//                         Child(
//                             Name("td").and(Class("trow1").or(Class("trow2"))),
//                             Name("strong")
//                         ),
//                         Name("span").and(Class("largetext"))
//                     )
//                 ).next().unwrap();
//                 let name = name_span.text();

//                 let hopefully_start = "https://community.tulpa.info/search.php?action=finduser&uid=";
//                 let url = find_button.attr("href").unwrap();
//                 if !url.starts_with(hopefully_start) {
//                     panic!();
//                 }
//                 let (_, user_id_str) = url.split_at(hopefully_start.len());
//                 let user_id:u64 = user_id_str.parse().unwrap();
//                 user = User::Known{id: user_id, name.to_string()};
//             }else{
//                 user = User::Anonymous;
//             }

//             let body_div = n.find(Class("post_body")).next().unwrap();
//             let content = body_div.text().trim();

//             /*let time_node = dbg!(child);
//             let time_node = dbg!(time_node.find(Name("tbody")).next());
//             let time_node = time_node.unwrap(); //tbody
//             let time_node = dbg!(time_node.children().filter(|e| e.is(Element)).nth(1));
//             let time_node = time_node.unwrap(); //2nd tr
//             let time_node = dbg!(time_node.children().filter(|e| e.is(Element)).next());
//             let time_node = time_node.unwrap(); //1st td of 2nd tr
//             let time_node = dbg!(time_node.find(Name("span").and(Class("smalltext"))).next());
//             let time_node = time_node.unwrap(); //time node*/
//             let time_node = n.find(Name("tbody")).next().unwrap() //tbody
//                 .children().filter(|e| e.is(Element)).nth(1).unwrap() //2nd tr
//                 .children().filter(|e| e.is(Element)).next().unwrap() //1st td of 2nd tr
//                 .find(Name("span").and(Class("smalltext"))).next().unwrap();
//             /*let time_node = n.find(Name("tr")).next().unwrap().next().unwrap() //tr
//                 .children().next().unwrap().find(Name("span").and(Class("smalltext"))).next().unwrap(); //smalltext span*/

//             //There are three possibilities:
//             //1. time_node has only text, in the format "YYYY-MM-DD, hh:MM"
//             //2. time_node looks like:
//             // <span class="smalltext"><span title="2019-10-21">Yesterday</span>, 12:48 </span>
//             // "Yesterday" can also be "Today"
//             //3. time_node looks like:
//             // <span class="smalltext"><span title="2019-10-22, 02:50">7 hours ago</span> </span>
//             // "hours" can also be "minutes" or "seconds" and may or may not be plural

//             fn recur(node: &select::node::Node, string: &mut String) {
//                 if let Some(text) = node.as_text() {
//                     string.push_str(text);
//                 } else if let Some(val) = node.attr("title") {
//                     string.push_str(val);
//                 } else {
//                     for child in node.children() {
//                         recur(&child, string);
//                     }
//                 }
//             }

//             let mut time_str = String::new();
//             recur(&time_node, &mut time_str);
//             //eprintln!("Parsing {}", time_str.trim());
//             let naive_time = chrono::NaiveDateTime::parse_from_str(time_str.trim(), "%Y-%m-%d, %H:%M").unwrap();
//             let post_time = chrono::DateTime::<chrono::Utc>::from_utc(naive_time, chrono::Utc);

//             Post{
//                 id: post_id,
//                 user,
//                 content: content.to_string(),
//                 posted_at: post_time,
//             }
//         }
//     }
// }
