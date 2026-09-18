use std::env;

use tokio_util::sync::CancellationToken;

use sublens::{
    live::format_summary,
    resolver::{Resolver, ResolverConfig},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("sublens=info")
        .init();
    let args: Vec<String> = env::args().collect();
    let Some(source) = args
        .windows(2)
        .find(|pair| pair[0] == "--url")
        .map(|pair| pair[1].clone())
    else {
        eprintln!("Usage: live_acceptance --url <subscription-url>");
        std::process::exit(2);
    };
    let resolver = Resolver::new(reqwest::Client::new(), ResolverConfig::default());
    match resolver.analyze(&source, CancellationToken::new()).await {
        Ok(report) => println!("{}", format_summary(&source, &report)),
        Err(error) => {
            eprintln!("URL fetched: FAIL");
            eprintln!("Analysis failed: {error}");
            std::process::exit(1);
        }
    }
}
