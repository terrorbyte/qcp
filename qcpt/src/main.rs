// qcp server - main entrypoint
// (c) 2024 Ross Younger

use clap::Parser as _;
use qcp::server::ServerArgs;
use tracing::{error, Level};

fn main() -> anyhow::Result<()> {
    let args = ServerArgs::parse();
    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_writer(std::io::stderr)
            .compact()
            .init();
    }

    qcp::server::main().map_err(|e| {
        error!("{e}");
        // TODO: Decide about error handling. For now detailed anyhow output is fine.
        e
    })
}
