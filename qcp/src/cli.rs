use crate::build_info;
/// qcp command line interface
/// (c) 2024 Ross Younger
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author,
    version(build_info::GIT_VERSION),
    about,
    long_about = "QUIC file copy utility"
)]
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
    // TODO!
    // This will become something scp-like.
    // qcp [options...] FILE [FILE...] SERVER:DESTDIR/DESTFILE
    // qcp [options...] SERVER:FILE DESTDIR/DESTFILE
}

/// Main CLI entrypoint
pub fn cli_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    Ok(())
}
