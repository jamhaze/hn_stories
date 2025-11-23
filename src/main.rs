use tokio;
use std::process;
use clap::{Args,Parser};

#[derive(Parser, Debug)]
#[command(
    version, 
    about = "This is a command line tool for retrieving the URLs of stories posted on https://news.ycombinator.com/", 
    long_about = None
)]
struct Cli {
    
    #[command(flatten)]
    find_by: FindBy,

    #[arg(
        short,
        long,
        default_value = "30", 
        help = "Set the limit for the number of stories you wish to retrieve with above options",
    )]
    limit: u8,

    #[arg(
        short,
        long,
        help = "Display the time at which the story was posted",
    )]
    time: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct FindBy {
    #[arg(
        short,
        long,
        value_parser = ["new", "top", "best"], 
        help = "The category of the stories to fetch",
    )]
    category: Option<String>,

    #[arg(
        short,
        long,
        help = "The query to search for stories with",
    )]
    query: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    if let Err(error) = hn_stories::run(cli.find_by.category, cli.find_by.query, cli.limit, cli.time).await {
        eprintln!("ERROR: {error}");
        process::exit(1);
    };
}
