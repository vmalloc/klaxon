use anyhow::Result;
use clap::{Parser, Subcommand};
use klaxon::{PdIssue, PdReporter};

#[derive(Parser)]
struct Config {
    #[clap(long, env = "PAGERDUTY_KEY")]
    pagerduty_key: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Trigger,
    Resolve,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let config = Config::parse();

    let mut pd_reporter = PdReporter::new(config.pagerduty_key)?;

    let issue = PdIssue {
        title: "Oh no".to_owned(),
        source: "Crate example".to_owned(),
        component: "Component name".to_owned(),
        dedup_fields: Default::default(),
    };

    match config.command {
        Command::Trigger => pd_reporter.trigger(issue),
        Command::Resolve => pd_reporter.resolve(issue),
    }

    pd_reporter.finish().await?;

    Ok(())
}
