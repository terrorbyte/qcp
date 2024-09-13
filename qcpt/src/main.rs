/// Transport utility for qcp - main entrypoint
/// (c) 2024 Ross Younger

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    qcpt::cli::cli_main()
}
