// qcp server command line interface
// (c) 2024 Ross Younger

use crate::build_info;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author,
    version(build_info::GIT_VERSION),
    about,
    long_about = "This is the QUIC file copy remote end. It is intended for unattended use. If you want to copy files, you should probably use qcp."
)]
#[command(styles=crate::styles::get())]
pub struct ServerArgs {
    /// Enable detailed debug output
    #[arg(short, long, action)]
    pub debug: bool,
}
