//! qcpt command line interface base
/// (c) 2024 Ross Younger
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = "QCP transport utility")]
//#[command(author(clap::crate_authors!()), version, about, long_about = "QCP transport utility")]
#[command(help_template(
    "\
{before-help}{name} {version}
(c) {author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
))]
#[command(styles=crate::styles::get())]
/// Top-level CLI definition
pub struct Cli {
    #[command(subcommand)]
    /// User's chosen subcommand
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
//#[command(flatten_help = true)]
/// Subcommands
pub enum Commands {
    Dummy,
}

/// Main CLI entrypoint
pub fn cli_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Dummy => Ok(()),
    }
}
