use tokio;
use std::process;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version, 
    about = "A program to retrieve the URLs of stories on hacker news via API", 
    long_about = None
)]
pub struct Args {
    
    #[arg(
        value_parser = ["new", "top", "best"], 
        default_value = "top", 
        help = "The category of the stories to fetch",
    )]
    story_cat: String,

    #[arg(
        default_value = "30", 
        help = "The number of stories you wish to retrieve for the above category",
    )]
    num_stories: u8,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = hn_stories::run(args.story_cat, args.num_stories.into()).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    };
}
