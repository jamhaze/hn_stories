use serde::Deserialize;
use serde_json::Value;
use reqwest::Client;
use futures::future::join_all;
use std::cmp::PartialEq;
use chrono::{DateTime, Local};

// The story struct.  Fields are title, url and time.  Time is a unix time.
#[derive(Deserialize, Debug)]
struct Story {
	#[serde(default = "return_na_string")]
	title: String,
    #[serde(default = "return_na_string")]
    url: String,
    #[serde(default)]
    time: i64,
}

// Used by the story struct to set the default value for title, url when no value is returned.
fn return_na_string() -> String {
    "N/A".to_string()
}


impl Story {

    // Defines how the story is displayed.  
    // If show_time is true, then it will also print the time converted into the local timezone.
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

    // Method to check if one story is equal to another.
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.url == other.url && self.time == other.time
    }
}

// This function is the only public function of the library.
// It should take the command line inputs supplied from main.rs and run the corresponding code.
pub async fn run(category: Option<String>, query: Option<String>, limit: u8, time: bool) -> Result<(), reqwest::Error> {

    let client = Client::new();

    // Convert the limit into a usize. 
    // This is required for it's use in the while loops of the below 'get' functions.
    let limit = limit.into();

    // Run the relevant function based on the arguments supplied.
    // Both functions will return Result<Vec<Story>, reqwest::Error>
    // Any error will be returned to the calling code.
    let mut stories = vec![];
    if let Some(c) = category {

        let firebase_base_url = String::from("https://hacker-news.firebaseio.com/v0");
        stories = get_stories_by_category(&client, c, limit, firebase_base_url).await?;

    } else if let Some(q) = query {

        let algolia_base_url = String::from("http://hn.algolia.com/api/v1");
        stories = get_stories_by_search(&client, q, limit, algolia_base_url).await?;
    }

    // For each story, use the show method.
    // The value of time, which is supplied by the user, determines whether the story's time will be shown.
    for story in stories {
        story.show(time);
    }

    // Return nothing and the execution will be complete if this point is reached.
    Ok(())
}

// Get the stories based on the category supplied (top, new, best)
async fn get_stories_by_category(
    client: &Client, 
    category: String, 
    limit: usize, 
    firebase_base_url: String,
) -> Result<Vec<Story>, reqwest::Error> {

    // Format the URL and make the request with client.
    // The JSON can be deserialised directly into a vector of i32, since the endpoint only has a single array of IDs.
    let stories_url = format!("{}/{}stories.json", firebase_base_url, category);
    let story_ids = client.get(stories_url).send().await?.json::<Vec<i32>>().await?;
    
    // Create an empty vector to store the async handles
    let mut handles = vec![];
    let mut i: usize = 0;
    
    // Continue the loop as long as i is less than the supplied limit and the returned list.
    while i < limit && i < story_ids.len() {
        if let Some(item) = story_ids.get(i) {

            // Subsitute the ID into the URL
           	let item_url = format!("{}/item/{}.json", firebase_base_url, item);

            // Using the async block to get the story here essentially allows new requests to be fired off
            // while we wait for the results of others to return.  This results in a faster execution.
			let handle = async {	     
				match get_story(&client, item_url).await {

                    // We want to handle the result of get_story internally, since if one requests fails,
                    // we can continue to the next one without crashing the whole program.
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

    // Join the handles to get the results.
    let joined = join_all(handles).await;

    // For each story, push it into a vector and return it.
    let mut stories = vec![];
    for story_opt in joined {
        if let Some(story) = story_opt {
            stories.push(story);
        }
    }
    Ok(stories)
}

// This function is used by get_stories_by_category
async fn get_story(client: &Client, item_url: String) -> Result<Story, reqwest::Error> {
    let resp = client.get(item_url).send().await?;
    let story = resp.json::<Story>().await?;
    Ok(story)
}

// Get the stories based on the search query supplied.
async fn get_stories_by_search(
    client: &Client, 
    query: String, 
    limit: usize, 
    algolia_base_url: String,
) -> Result<Vec<Story>, reqwest::Error> {
    
    // This API call returns a more complex JSON than the ones in get_stories_by_category, so
    // it's easier to use the get_json_from_str function to return a serde_json::Value type.
    let search_url = format!("{}/search?query={}&tags=story", algolia_base_url, query);
    let resp = client.get(search_url).send().await?;
    let json_text = resp.text().await?;
    let result = get_json_from_str(&json_text);

    let mut stories = vec![];
    match result {

        Ok(value) => {

            // If the result of get_json_from_str is OK, it should contain a field called hits
            // which contains an array of stories which satisfied the search query.
            if let Some(hits) = value["hits"].as_array() {

                // Get each "hit" and make a story with the Story struct.  
                // Loop until either the limit or the end of the array is reached.
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

        // There will be an error if get_json_from_str results in a serde_json::Error
        Err(err) => eprintln!("ERROR: {err}"),
    }
    Ok(stories)
}

fn get_json_from_str(text: &str) -> Result<Value, serde_json::Error> {
    let value: Value = serde_json::from_str(text)?;
    Ok(value)
}

// The below are tests which work by setting up a mock server which returns JSON
// to be used by the functions.
#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serde_json::json;
    use super::*;

    #[tokio::test]
    async fn test_get_story_by_category() {

        let test_story_1 = Story {
            time: 0000000000,
            title: String::from("This is the first test story"),
			url: String::from("https://www.test1.com"),
        };
        let test_story_2 = Story {
            time: 0000000001,
            title: String::from("This is the second test story"),
			url: String::from("https://www.test2.com"),
        };

        let test_story_vec = vec![test_story_1, test_story_2];

        let server = MockServer::start();
        let _mock_cat = server.mock(|when, then| {
			when.method(GET)
				.path("/topstories.json");
			then.status(200)
				.json_body(json!(
                    [10000000, 10000001]
                ));
        });

        let _mock_item_1 = server.mock(|when, then| {
			when.method(GET)
				.path("/item/10000000.json");
			then.status(200)
				.json_body(json!({
                    "by": "me",
                    "descendants": 0,
                    "id": 0,
                    "kids": [],
                    "score": 0,
                    "time": 0000000000,
                    "title": "This is the first test story",
                    "type": "story",
                    "url": "https://www.test1.com"
                }));
        });

        let _mock_item_2 = server.mock(|when, then| {
			when.method(GET)
				.path("/item/10000001.json");
			then.status(200)
				.json_body(json!({
                    "by": "me",
                    "descendants": 0,
                    "id": 0,
                    "kids": [],
                    "score": 0,
                    "time": 0000000001,
                    "title": "This is the second test story",
                    "type": "story",
                    "url": "https://www.test2.com"
                }));
        });

        let client = Client::new();
        let category = String::from("top");
        let limit: usize = 2;
        let base_url = server.base_url();

        let result = get_stories_by_category(&client, category, limit, base_url).await;

		assert_eq!(result.unwrap(), test_story_vec);
    }

    /*
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
    */
}
