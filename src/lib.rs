use serde::Deserialize;
use reqwest::Client;
use futures::future::join_all;

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
    
    let mut handles = vec![];
    let mut i: usize = 0;
    while i < num_stories && i < story_ids.len() {
        
        if let Some(item) = story_ids.get(i) {
            let handle = async {
                match get_story(&client, item).await {
                    Ok(story) => {
                        println!("title : {}\nurl   : {}\n", story.title, story.url);
                    }
                    Err(error) => {
                        println!("error : {}\n", error);
                    }
                }
                
            };
            handles.push(handle);
        }
        i += 1;
    }

    let _ = join_all(handles).await;
    
    Ok(())
}

async fn get_story(client: &Client, item: &i32) -> Result<Story, reqwest::Error> {
    let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", item);
    Ok(client.get(item_url).send().await?.json::<Story>().await?)
}