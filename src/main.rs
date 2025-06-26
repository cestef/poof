use clap::Parser;
use poof::{cli::Opts, core::run};

#[tokio::main]
async fn main() -> miette::Result<()> {
    Ok(run(Opts::parse()).await?)
}
