use clap::{command, Parser};
use futures::{stream::FuturesUnordered, StreamExt};
use std::{collections::HashMap, error::Error, time::Duration};

#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<Stats>>();

    let raw_urls = match (args.file.as_ref(), args.url) {
        (Some(file), None) => {
            let file = std::fs::read_to_string(file)?;
            file.lines()
                .filter_map(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                })
                .collect()
        }
        (None, Some(url)) => vec![url],
        _ => {
            eprintln!("Either file or url should be provided");
            std::process::exit(1);
        }
    };

    let urls = {
        let raw_url_len = raw_urls.len();
        let repeat_factor = if raw_url_len < args.number_of_requests && args.file.is_some() {
            args.number_of_requests / raw_url_len
        } else {
            1
        };
        raw_urls
            .into_iter()
            .cycle()
            .take(repeat_factor * raw_url_len)
            .collect::<Vec<_>>()
    };

    let mut tasks = Vec::with_capacity(args.concurrency);

    for _ in 0..args.concurrency {
        tasks.push(tokio::spawn(handle_concurrent_requests(
            urls.clone(),
            args.number_of_requests,
            sender.clone(),
        )));
    }

    drop(sender);
    for task in tasks {
        task.await?;
    }
    compute_statistics(receiver).await;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short)]
    url: Option<String>,
    #[arg(short, default_value_t = 10)]
    number_of_requests: usize,
    #[arg(short, default_value_t = 1)]
    concurrency: usize,
    #[arg(short)]
    file: Option<String>,
}

struct Stats {
    url: String,
    status_code: StatusCode,
    duration: std::time::Duration,
    ttfb: std::time::Duration,
    ttlb: std::time::Duration,
}

enum StatusCode {
    Success,
    ClientError,
    ServerError,
}

async fn handle_concurrent_requests(
    urls: Vec<String>,
    number_of_requests: usize,
    sender: tokio::sync::mpsc::UnboundedSender<Vec<Stats>>,
) {
    let mut futures = urls
        .into_iter()
        .map(|url| send_url_requests(url, number_of_requests))
        .collect::<FuturesUnordered<_>>();

    while let Some(result) = futures.next().await {
        sender.send(result).unwrap();
    }
    drop(sender);
}

async fn send_url_requests(url: String, number_of_requests: usize) -> Vec<Stats> {
    let mut results = Vec::with_capacity(number_of_requests);
    for _ in 0..number_of_requests {
        let stat = send_single_request(url.clone()).await;
        results.push(stat);
    }
    results
}

async fn send_single_request(url: String) -> Stats {
    let start = std::time::Instant::now();
    let response = reqwest::get(&url).await;
    let (ttfb, ttlb, status_code) = if let Ok(response) = response {
        let status = response.status();
        let status_code = if status.is_success() {
            StatusCode::Success
        } else if status.is_client_error() {
            StatusCode::ClientError
        } else {
            StatusCode::ServerError
        };
        let ttfb = start.elapsed();
        let _ = response.bytes().await;
        let ttlb = start.elapsed();
        (ttfb, ttlb, status_code)
    } else {
        (start.elapsed(), start.elapsed(), StatusCode::ServerError)
    };

    Stats {
        url,
        status_code,
        duration: start.elapsed(),
        ttfb,
        ttlb,
    }
}

async fn compute_statistics(mut receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<Stats>>) {
    let mut url_stats: HashMap<String, UrlStatistics> = std::collections::HashMap::new();

    while let Some(stats) = receiver.recv().await {
        for stat in stats {
            url_stats
                .entry(stat.url.clone())
                .or_default()
                .add_stat(&stat);
        }
    }
    url_stats.into_iter().for_each(|(url, stats)| {
        println!("Results for {}", url);
        stats.print_output();
        println!("-------------------------------");
    });
}

#[derive(Default)]
struct UrlStatistics {
    all_requests: usize,
    success: usize,
    client_errors: usize,
    server_errors: usize,

    min_duration: Duration,
    max_duration: Duration,
    total_duration: Duration,

    total_ttfb: Duration,
    min_ttfb: Duration,
    max_ttfb: Duration,

    total_ttlb: Duration,
    min_ttlb: Duration,
    max_ttlb: Duration,
}

impl UrlStatistics {
    pub fn add_stat(&mut self, stat: &Stats) {
        self.all_requests += 1;
        match stat.status_code {
            StatusCode::Success => self.success += 1,
            StatusCode::ClientError => self.client_errors += 1,
            StatusCode::ServerError => self.server_errors += 1,
        }

        self.max_duration = self.max_duration.max(stat.duration);
        self.min_duration = self.min_duration.min(stat.duration);
        self.total_duration += stat.duration;

        self.total_ttfb += stat.ttfb;
        self.min_ttfb = self.min_ttfb.min(stat.ttfb);
        self.max_ttfb = self.max_ttfb.max(stat.ttfb);

        self.total_ttlb += stat.ttlb;
        self.min_ttlb = self.min_ttlb.min(stat.ttlb);
        self.max_ttlb = self.max_ttlb.max(stat.ttlb);
    }

    pub fn print_output(self) {
        println!(
            " Total Requests (2XX).......................: {}",
            self.success
        );
        println!(
            " Failed Requests (4XX)...................: {}",
            self.client_errors
        );
        println!(
            " Failed Requests (5XX)...................: {}",
            self.server_errors
        );
        println!(
            " Total Request Time (s) (Min, Max, Mean).....: {:.2}, {:.2}, {:.2}",
            self.min_duration.as_secs_f64(),
            self.max_duration.as_secs_f64(),
            self.total_duration.as_secs_f64() / self.all_requests as f64
        );
        println!(
            " Time to First Byte (s) (Min, Max, Mean).....: {:.2}, {:.2}, {:.2}",
            self.min_ttfb.as_secs_f64(),
            self.max_ttfb.as_secs_f64(),
            self.total_ttfb.as_secs_f64() / self.all_requests as f64
        );
        println!(
            " Time to Last Byte (s) (Min, Max, Mean)......: {:.2}, {:.2}, {:.2}",
            self.min_ttlb.as_secs_f64(),
            self.max_ttlb.as_secs_f64(),
            self.total_ttlb.as_secs_f64() / self.all_requests as f64
        );
    }
}
