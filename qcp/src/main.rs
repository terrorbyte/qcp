// qcp utility - main entrypoint
// (c) 2024 Ross Younger
use clap::Parser as _;
use qcp::client::ClientArgs;

use tracing::{error, Level};

fn main() -> anyhow::Result<()> {
    let args = ClientArgs::parse();
    let trace_level = match args.debug {
        true => Level::TRACE,
        false => match args.quiet {
            true => Level::ERROR,
            false => Level::INFO,
        },
    };
    tracing_subscriber::fmt()
        .with_max_level(trace_level)
        .compact()
        .init();

    qcp::client::main(&args).map_err(|e| {
        error!("{e}");
        // TODO: Decide about error handling. For now detailed anyhow output is fine.
        e
    })
}
