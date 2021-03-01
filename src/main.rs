use serde_derive::Deserialize;

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

fn main() {
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
