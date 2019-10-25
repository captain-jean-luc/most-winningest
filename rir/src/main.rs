/*extern crate reqwest;
extern crate select;
extern crate futures;*/

//use std::collections::HashMap;

//use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = std::env::var("MW_PASSWORD").unwrap();
    
    
    let cli = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .gzip(true)
        .referer(false)
        .timeout(core::time::Duration::from_secs(10))
        .build()?;

    let login_get_url = "https://community.tulpa.info/member.php?action=login";
    let login_get_resp = cli.get(login_get_url).send().await.unwrap();
    dbg!(login_get_resp.status());
    let document = Document::from(login_get_resp.text().await.unwrap().as_str());
    let login_form = document.find(Name("form").and(Attr("action","member.php"))).next().unwrap();
    //get fields action, url, and my_post_key
    let mut fields = login_form.find(Name("input").and(Attr("type","hidden")))
        .map(|node| (node.attr("name").unwrap_or(""), node.attr("value").unwrap_or("")))
        .collect::<Vec<_>>();
    fields.push(("username","jean-luc-bot"));
    fields.push(("password",&password));
    fields.push(("remember","yes"));
    fields.push(("submit","Login"));

    let login_post = cli.post("https://community.tulpa.info/member.php").form(&fields).send().await?;
    assert!(login_post.status().is_success());
    //we don't actually care about the content, but we do want to wait for .info to finish sending the response

    let _ = login_post.bytes().await;

    let first_page_text = cli.get("https://community.tulpa.info/thread-game-last-one-to-post-wins").send().await.unwrap().text().await.unwrap();
    let document = Document::from(first_page_text.as_str());
    let posts_node = document.find(Attr("id","posts")).next().unwrap();
    for child in posts_node.children() {
        
    }
    

    Ok(())
}
