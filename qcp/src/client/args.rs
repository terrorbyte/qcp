// qcp client - command line interface
// (c) 2024 Ross Younger
use crate::build_info;
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
/// The arguments we need to set up a client
pub struct ClientArgs {
    /// Enable detailed debug output
    #[arg(short, long, action)]
    pub debug: bool,
    /// Quiet mode (reduced chatter)
    #[arg(short, long, action)]
    pub quiet: bool,
    /// Connection timeout (seconds)
    #[arg(short, long, default_value("1"))]
    pub timeout: u16,
    /// Enables server debug output
    #[arg(short, long, action)]
    pub server_debug: bool,
    // TODO!
    // This will become something scp-like.
    // qcp [options...] FILE [FILE...] SERVER:DESTDIR/DESTFILE
    // qcp [options...] SERVER:FILE DESTDIR/DESTFILE
}
