use tokio;
use std::process;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version, 
    about = "This is a command line tool for retrieving the URLs of stories posted on https://news.ycombinator.com/", 
    long_about = None
)]
struct Args {
    
    #[arg(
        short = 'c',
        long = "category",
        value_parser = ["new", "top", "best"], 
        default_value = "top", 
        help = "The category of the stories to fetch",
    )]
    story_cat: String,

    #[arg(
        short,
        long,
        default_value = "30", 
        help = "Set the limit for the number of stories you wish to retrieve for the above category",
    )]
    limit: u8,

    #[arg(
        short = 't',
        long = "time",
        help = "Display the time at which the story was posted",
    )]
    show_time: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(error) = hn_stories::run(args.story_cat, args.limit, args.show_time).await {
        eprintln!("ERROR: {error}");
        process::exit(1);
    };
}
