// qcp utility - main entrypoint
// (c) 2024 Ross Younger
use clap::Parser as _;
use qcp::client::ClientArgs;

fn main() -> anyhow::Result<()> {
    let args = ClientArgs::parse();
    let trace_level = match args.debug {
        true => "trace",
        false => match args.quiet {
            true => "error",
            false => "info",
        },
    };
    qcp::util::setup_tracing(trace_level)?;

    qcp::client::main(&args).map_err(|e| {
        tracing::error!("{e}");
        // TODO: Decide about error handling. For now detailed anyhow output is fine.
        e
    })
}
