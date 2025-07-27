use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    #[serde(default)]
    url: String,
}

pub async fn run(story_cat: String, num_stories: usize) -> Result<(), reqwest::Error> {
    
    let client = Client::new();
    let stories_url = format!("https://hacker-news.firebaseio.com/v0/{}stories.json", story_cat);
    let story_ids = client.get(stories_url).send().await?.json::<Vec<i32>>().await?;
    
    let mut i: usize = 0;
    while i < num_stories && i < story_ids.len() {
        if let Some(item) = story_ids.get(i) {
            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", item);
            let story = client.get(item_url).send().await?.json::<Story>().await?;
            println!("title : {}\nurl   : {}\n", story.title, story.url);
        }
        i += 1;
    }
    
    println!("You requested {} stories from the {}stories page and {} were retrieved\n", num_stories, story_cat, i);
    Ok(())
}