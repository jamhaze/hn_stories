use reqwest::Client;
use tokio;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    #[serde(default)]
    url: String,
}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    
    let client = Client::new();
    let top_stories_url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let story_ids = client.get(top_stories_url).send().await?.json::<Vec<i32>>().await?;
    for id in &story_ids[..5] {
        let story = get_story(&client, id).await?;
        println!("title : {}\nurl   : {}\n", story.title, story.url);
    }
    Ok(())
}

async fn get_story(client: &Client, id: &i32) -> Result<Story, reqwest::Error> {
    let story_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id.to_string());
    client.get(story_url).send().await?.json::<Story>().await
}
