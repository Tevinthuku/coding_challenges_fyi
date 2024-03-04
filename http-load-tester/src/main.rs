use clap::{command, Parser};
use std::error::Error;

#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Vec<RequestResult>>(args.concurrency);

    let mut tasks = Vec::with_capacity(args.concurrency);
    for _ in 0..args.concurrency {
        tasks.push(tokio::spawn(handle_concurrent_requests(
            args.url.clone(),
            args.number_of_requests,
            sender.clone(),
        )));
    }

    drop(sender);

    let mut success = 0;
    let mut failure = 0;

    while let Some(request_results) = receiver.recv().await {
        for result in request_results {
            match result {
                RequestResult::Success => success += 1,
                RequestResult::Error => failure += 1,
            }
        }
    }

    for task in tasks {
        task.await?;
    }
    println!("Successes: {}", success);
    println!("Failure: {}", failure);
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short)]
    url: String,
    #[arg(short, default_value_t = 10)]
    number_of_requests: usize,
    #[arg(short, default_value_t = 1)]
    concurrency: usize,
}

async fn handle_concurrent_requests(
    url: String,
    number_of_requests: usize,
    sender: tokio::sync::mpsc::Sender<Vec<RequestResult>>,
) {
    let mut results = Vec::with_capacity(number_of_requests);
    for _ in 0..number_of_requests {
        let response = reqwest::get(&url)
            .await
            .and_then(|data| data.error_for_status());
        results.push(match response {
            Ok(_) => RequestResult::Success,
            Err(_) => RequestResult::Error,
        });
    }
    sender.send(results).await.unwrap();
    drop(sender);
}

enum RequestResult {
    Success,
    Error,
}
