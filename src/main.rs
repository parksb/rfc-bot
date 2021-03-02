use serde_derive::Deserialize;
use std::fs::File;
use std::io::Write;
use std::{env, fs};

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

#[derive(Deserialize, Debug)]
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
    const PATH: &str = "last_build_date";
    if fs::read_to_string(PATH).unwrap() == last_build_date {
        true
    } else {
        let mut file = File::create(PATH).unwrap();
        file.write_all(last_build_date.as_bytes()).unwrap();
        false
    }
}

#[tokio::main]
async fn main() {
    let Channel {
        last_build_date,
        items,
    } = parse();

    if !is_updated(&last_build_date) {
        let text = compose_text(&items[0]);
        tweet(text).await;
    }
}
