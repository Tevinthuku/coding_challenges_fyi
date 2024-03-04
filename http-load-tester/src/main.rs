use clap::{command, Parser};
use std::error::Error;

#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    for _ in 0..args.number_of_requests {
        let response = reqwest::get(&args.url).await?;
        println!("Response code: {}", response.status().as_u16());
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short)]
    url: String,
    #[arg(short, default_value_t = 1)]
    number_of_requests: u8,
}
