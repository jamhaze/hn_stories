use serde::Deserialize;
use serde_json::Value;
use reqwest::Client;
use futures::future::join_all;
use std::cmp::PartialEq;
use chrono::{DateTime, Local};


#[derive(Deserialize, Debug)]
struct Story {
	#[serde(default = "return_na_string")]
	title: String,
    #[serde(default = "return_na_string")]
    url: String,
    #[serde(default)]
    time: i64,
}

fn return_na_string() -> String {
    "N/A".to_string()
}

impl Story {
    fn show(&self, show_time: bool) {
        println!("Title : {}", self.title);
        println!("URL   : {}", self.url);
        if show_time {
    		let mut local_dt_string = String::from("Unknown");
	    	if let Some(dt) = DateTime::from_timestamp(self.time, 0) {
                local_dt_string = dt.with_timezone(&Local).to_string();
		    }
            println!("Time  : {local_dt_string}");
        }
        println!();
    }
}

impl PartialEq for Story {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.url == other.url && self.time == other.time
    }
}

pub async fn run(category: Option<String>, query: Option<String>, limit: u8, time: bool) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let limit = limit.into();
    let mut stories = vec![];
    if let Some(c) = category {
        stories = get_stories_by_category(&client, c, limit).await?;
    } else if let Some(q) = query {
        stories = get_stories_by_search(&client, q, limit).await?;
    }
    for story in stories {
        story.show(time);
    }
    Ok(())
}

async fn get_stories_by_category(client: &Client, category: String, limit: usize) -> Result<Vec<Story>, reqwest::Error> {
    let stories_url = format!("https://hacker-news.firebaseio.com/v0/{}stories.json", category);
    let story_ids = client.get(stories_url).send().await?.json::<Vec<i32>>().await?;
    
    let mut handles = vec![];
    let mut i: usize = 0;
    
    while i < limit && i < story_ids.len() {
        if let Some(item) = story_ids.get(i) {
           	let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", item);
			let handle = async {	     
				match get_story(&client, item_url).await {
                    Ok(story) => Some(story),
                    Err(err) => {
                        eprintln!("ERROR: {err}");
                        None
                    }
                } 
            };
            handles.push(handle);
        }
        i += 1;
    }
    let joined = join_all(handles).await;

    let mut stories = vec![];
    for story_opt in joined {
        if let Some(story) = story_opt {
            stories.push(story);
        }
    }
    Ok(stories)
}

async fn get_story(client: &Client, item_url: String) -> Result<Story, reqwest::Error> {
    let resp = client.get(item_url).send().await?;
    let story = resp.json::<Story>().await?;
    Ok(story)
}

async fn get_stories_by_search(client: &Client, query: String, limit: usize) -> Result<Vec<Story>, reqwest::Error> {
    
    let search_url = format!("http://hn.algolia.com/api/v1/search?query={}&tags=story", query);
    let resp = client.get(search_url).send().await?;
    let json_text = resp.text().await?;
    let result = get_json_from_str(&json_text);
    let mut stories = vec![];
    match result {
        Ok(value) => {
            if let Some(hits) = value["hits"].as_array() {
                let mut i: usize = 0;
                while i < limit && i < hits.len() {
                    let hit = &hits[i];
                    if let (Some(h_time), Some(h_title), Some(h_url)) = 
                    (hit["created_at_i"].as_i64(), 
                    hit["title"].as_str(), 
                    hit["url"].as_str()) {
                        stories.push(Story {
                            time: h_time,
                            title: String::from(h_title),
                            url: String::from(h_url),
                        });
                    }
                    i += 1;
                }
            }
        }
        Err(err) => eprintln!("ERROR: {err}"),
    }
    Ok(stories)
}

fn get_json_from_str(text: &str) -> Result<Value, serde_json::Error> {
    let value: Value = serde_json::from_str(text)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serde_json::json;
    use super::*;
	
    #[tokio::test]
	async fn test_get_story() {	
		let client = Client::new();
		let test_story = Story {
			time: 0000000000,
			title: String::from("This is a test"),
			url: String::from("https://www.test.com"),	
        };
		let server = MockServer::start();
		let mock = server.mock(|when, then| {
			when.method(GET)
				.path("/v0/item/10000000.json");
			then.status(200)
				.json_body(json!({
                    "by": "me",
                    "descendants": 0,
                    "id": 0,
                    "kids": [10000001],
                    "score": 0,
                    "time": 0000000000,
                    "title": "This is a test",
                    "type": "story",
                    "url": "https://www.test.com"
                }));
        });
		let result = get_story(&client, server.url("/v0/item/10000000.json")).await;
        
		mock.assert();
        assert_eq!(result.unwrap(), test_story);
			
	}

    #[tokio::test]
    async fn test_no_url() {
        let client = Client::new();
        let test_story = Story {
			time: 0000000000, 
            title: String::from("This is a test"),
            url: String::from("N/A"),
        };
		let server = MockServer::start();
        let mock = server.mock(|when, then| {
			when.method(GET)
				.path("/v0/item/10000000.json");
			then.status(200)
				.json_body(json!({
                    "by": "me",
                    "descendants": 0,
                    "id": 0,
                    "kids": [10000001],
                    "score": 0,
                    "time": 0000000000,
                    "title": "This is a test",
                    "type": "story",
                }));
        });
		let result = get_story(&client, server.url("/v0/item/10000000.json")).await;
        
		mock.assert();
		assert_eq!(result.unwrap(), test_story);
    }
}
