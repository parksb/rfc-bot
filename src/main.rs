use serde_derive::Deserialize;
use std::fs::File;
use std::io::Write;
use std::{env, fs, thread, time};

#[derive(Deserialize, Debug)]
struct Rss {
    channel: Channel,
}

#[derive(Deserialize, Debug)]
struct Channel {
    #[serde(rename = "lastBuildDate")]
    last_build_date: String,
    #[serde(rename = "item")]
    items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone)]
struct Item {
    title: String,
    link: String,
    description: String,
}

async fn tweet(text: String) {
    dotenv::dotenv().unwrap();
    let consumer_key = env::var("CONSUMER_KEY").unwrap();
    let consumer_secret = env::var("CONSUMER_SECRET").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();
    let access_secret = env::var("ACCESS_SECRET").unwrap();

    let consumer = egg_mode::KeyPair::new(consumer_key, consumer_secret);
    let access = egg_mode::KeyPair::new(access_token, access_secret);
    let token = egg_mode::Token::Access { consumer, access };

    egg_mode::tweet::DraftTweet::new(text)
        .send(&token)
        .await
        .unwrap();
}

fn parse() -> Channel {
    const URL: &str = "https://www.rfc-editor.org/rfcrss.xml";
    let res = reqwest::blocking::get(URL).unwrap();
    let xml = res.text().unwrap();
    serde_xml_rs::from_str::<Rss>(&xml).unwrap().channel
}

fn compose_text(item: &Item) -> String {
    const MAX_LENGTH: usize = 280;
    const ELLIPSIS: &str = "...";
    let text = format!("[{}]({}) {}", item.title, item.link, item.description);
    if text.len() > MAX_LENGTH {
        let text = text[0..(MAX_LENGTH - ELLIPSIS.len())].trim();
        format!("{}{}", text, ELLIPSIS)
    } else {
        text
    }
}

fn is_updated(last_build_date: &str) -> bool {
    let current_exe = env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let path = format!("{}/last_build_date", current_dir.display());
    if fs::read_to_string(&path).unwrap() == last_build_date {
        true
    } else {
        let mut file = File::create(&path).unwrap();
        file.write_all(last_build_date.as_bytes()).unwrap();
        false
    }
}

fn slice_items(items: &[Item]) -> Vec<Item> {
    let current_exe = env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let path = format!("{}/last_link", current_dir.display());

    let last_link = fs::read_to_string(&path);
    let lastest_link = &items.first().unwrap().link;

    let mut file = File::create(&path).unwrap();
    file.write_all(lastest_link.as_bytes()).unwrap();

    if let Ok(last_link) = last_link {
        let limit = items.iter().position(|item| item.link.trim() == last_link.trim()).unwrap();
        items[0..limit].to_vec()
    } else {
        (*items).to_vec()
    }
}

#[tokio::main]
async fn main() {
    let Channel {
        last_build_date,
        items,
    } = parse();

    if !is_updated(&last_build_date) {
        for item in slice_items(&items).into_iter().rev().collect::<Vec<Item>>() {
            let text = compose_text(&item);
            let delay = time::Duration::from_secs(60);
            tweet(text).await;
            thread::sleep(delay);
        }
    }
}

#[cfg(test)]
impl Item {
    fn new(title: &str, link: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            link: link.to_string(),
            description: description.to_string(),
        }
    }
}

#[test]
fn test_compose_text_with_short_item() {
    let title = "RFC 3092: Etymology of Foo";
    let link = "https://www.rfc-editor.org/info/rfc3092";
    let description = "This memo provides information for the Internet community.";

    let actual = compose_text(&Item::new(title, link, description));
    let expected = format!("[{}]({}) {}", title, link, description);

    assert_eq!(actual, expected);
}

#[test]
fn test_compose_text_with_long_item() {
    let title = "RFC 5321: Simple Mail Transfer Protocol";
    let link = "https://www.rfc-editor.org/info/rfc5321";
    let description = "This document is a specification of the basic protocol for Internet electronic mail transport. It consolidates, updates, and clarifies several previous documents, making all or parts of most of them obsolete.";

    let actual = compose_text(&Item::new(title, link, description));
    let expected = format!("[{}]({}) {}...", title, link, &description[0..193]);

    assert_eq!(actual, expected);
}
