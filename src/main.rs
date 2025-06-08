use std::error::Error;
use serde::Deserialize;
use reqwest::blocking;

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    #[serde(default)]
    url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ts_url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let resp = blocking::get(ts_url)?;
    
    let story_ids: Vec<i32> = resp.json()?;
    let story_endpoints: Vec<String> = story_ids.iter().map(|x| {
        let id_string = x.to_string();
        format!("https://hacker-news.firebaseio.com/v0/item/{}.json", &id_string)
    }).collect();

    for url in &story_endpoints[..30] {
        let resp = blocking::get(url)?;
        let story: Story = resp.json()?;
        println!("title : {}", story.title);
        println!("url   : {}\n", story.url);
    }

    Ok(())
}
