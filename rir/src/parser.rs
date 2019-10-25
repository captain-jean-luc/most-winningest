use chrono::Utc;
use select::node::Node;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name, Child, Element};

#[derive(Debug)]
pub struct Page {
    pub posts: Vec<Post>,
}

impl From<Document> for Page {
    fn from(document: Document) -> Page {
        let posts = Vec::new();
        for node in document.find(Name("table")){
            if let Some(val) = node.attr("id") {
                if val.starts_with("post_") {
                    posts.push(Post::from(node));
                }
            }
        }
        Page{
            posts,
        }
    }

#[derive(Debug)]
pub enum User {
    Anonymous,
    Known{id: u64, name: String},
}

#[derive(Debug)]
pub struct Post {
    pub id:u64,
    pub user:User,
    pub post_content:String,
    pub posted_at:chrono::DateTime<Utc>,
}

impl From<Node<'_>> for Post {
    fn from(n: Node<'_>) -> Post {
        if n.name() == Some("table") {
            let node_id = n.attr("id").unwrap();
            if !node_id.starts_with("post_") {
                panic!();
            }
            let (_, post_id_str) = node_id.split_at(5);
            let post_id:u64 = post_id_str.parse().unwrap();
            let user:User;
            if let Some(find_button) = n.find(Class("postbit_find")).next() {
                let name_span = n.find(
                    Child(
                        Child(
                            Name("td").and(Class("trow1").or(Class("trow2"))),
                            Name("strong")
                        ),
                        Name("span").and(Class("largetext"))
                    )
                ).next().unwrap();
                let name = name_span.text();

                let hopefully_start = "https://community.tulpa.info/search.php?action=finduser&uid=";
                let url = find_button.attr("href").unwrap();
                if !url.starts_with(hopefully_start) {
                    panic!();
                }
                let (_, user_id_str) = url.split_at(hopefully_start.len());
                let user_id:u64 = user_id_str.parse().unwrap();
                user = User::Known{id: user_id, name.to_string()};
            }else{
                user = User::Anonymous;
            }

            let body_div = n.find(Class("post_body")).next().unwrap();
            let content = body_div.text().trim();

            /*let time_node = dbg!(child);
            let time_node = dbg!(time_node.find(Name("tbody")).next());
            let time_node = time_node.unwrap(); //tbody
            let time_node = dbg!(time_node.children().filter(|e| e.is(Element)).nth(1));
            let time_node = time_node.unwrap(); //2nd tr
            let time_node = dbg!(time_node.children().filter(|e| e.is(Element)).next());
            let time_node = time_node.unwrap(); //1st td of 2nd tr
            let time_node = dbg!(time_node.find(Name("span").and(Class("smalltext"))).next());
            let time_node = time_node.unwrap(); //time node*/
            let time_node = n.find(Name("tbody")).next().unwrap() //tbody
                .children().filter(|e| e.is(Element)).nth(1).unwrap() //2nd tr
                .children().filter(|e| e.is(Element)).next().unwrap() //1st td of 2nd tr
                .find(Name("span").and(Class("smalltext"))).next().unwrap();
            /*let time_node = n.find(Name("tr")).next().unwrap().next().unwrap() //tr
                .children().next().unwrap().find(Name("span").and(Class("smalltext"))).next().unwrap(); //smalltext span*/

            //There are three possibilities:
            //1. time_node has only text, in the format "YYYY-MM-DD, hh:MM"
            //2. time_node looks like:
            // <span class="smalltext"><span title="2019-10-21">Yesterday</span>, 12:48 </span>
            // "Yesterday" can also be "Today"
            //3. time_node looks like:
            // <span class="smalltext"><span title="2019-10-22, 02:50">7 hours ago</span> </span>
            // "hours" can also be "minutes" or "seconds" and may or may not be plural

            fn recur(node: &select::node::Node, string: &mut String) {
                if let Some(text) = node.as_text() {
                    string.push_str(text);
                } else if let Some(val) = node.attr("title") {
                    string.push_str(val);
                } else {
                    for child in node.children() {
                        recur(&child, string);
                    }
                }
            }

            let mut time_str = String::new();
            recur(&time_node, &mut time_str);
            //eprintln!("Parsing {}", time_str.trim());
            let naive_time = chrono::NaiveDateTime::parse_from_str(time_str.trim(), "%Y-%m-%d, %H:%M").unwrap();
            let post_time = chrono::DateTime::<chrono::Utc>::from_utc(naive_time, chrono::Utc);

            Post{
                id: post_id,
                user,
                content: content.to_string(),
                posted_at: post_time,
            }
        }
    }
}
