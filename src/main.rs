use serde_derive::Deserialize;
use std::env;

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

#[tokio::main]
async fn main() {
    let url = "https://www.rfc-editor.org/rfcrss.xml";
    let res = reqwest::blocking::get(url).unwrap();
    let xml = res.text().unwrap();

    let Channel {
        last_build_date,
        items,
    } = serde_xml_rs::from_str::<Rss>(&xml).unwrap().channel;

    println!("{:?}", last_build_date);
    println!("{:?}", items);
}
