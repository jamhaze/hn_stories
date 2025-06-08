use std::error::Error;
use std::thread;
use serde::Deserialize;
use reqwest::blocking;

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    #[serde(default)]
    url: String,
}

fn get_stories(cat: &str, id: &i32) -> Result<(), Box<dyn Error>> {
    let id_string = id.to_string();
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", &id_string);
    let resp = blocking::get(url)?;
    let story: Story = resp.json()?;
    println!("cat   : {}\ntitle : {}\nurl   : {}\n", cat, story.title, story.url);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let ts_url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let resp = blocking::get(ts_url)?;
    let ts_ids: Vec<i32> = resp.json()?;

    let handle = thread::spawn(move || {
        for id in &ts_ids[..15] {
            let _ = get_stories("top", id);
        }
    });

    let ns_url = "https://hacker-news.firebaseio.com/v0/newstories.json";
    let resp = blocking::get(ns_url)?;
    let ns_ids: Vec<i32> = resp.json()?;

    for id in &ns_ids[..15] {
        let _ = get_stories("new", id);
    }
    
    handle.join().unwrap();

    Ok(())
}
