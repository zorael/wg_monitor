//! Command-line interface (CLI) definitions and parsing for the program.
//!
//! The members of the `Cli` struct correspond to the command-line arguments
//! that the program accepts, and the `clap` crate is used to parse these
//! arguments when the program is run.
//!
//! The `///` comments above each member of the `Cli` struct become the
//! help text for the corresponding command-line argument.

use clap::Parser;

use crate::defaults;

#[derive(Parser)]
#[command(name = defaults::program_metadata::PROGRAM_NAME)]
#[command(author = defaults::program_metadata::AUTHORS)]
//#[command(version = defaults::program_metadata::VERSION)]
pub struct Cli {
    /// Specify an alternate configuration directory
    #[arg(short = 'c', long, value_name = "path")]
    pub config_dir: Option<String>,

    /// Word the first notification as if the program was not just started
    #[arg(long)]
    pub resume: bool,

    /// Skip the first run and thus the first notification
    #[arg(long)]
    pub skip_first: bool,

    /// Disable timestamps in terminal output
    #[arg(long)]
    pub disable_timestamps: bool,

    /// Output configuration to screen and exit
    #[arg(long)]
    pub show: bool,

    /// Print some additional information
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Print much more additional information
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Perform a dry run, echoing what would be done
    #[arg(long)]
    pub dry_run: bool,

    /// Write configuration to disk
    #[arg(long)]
    pub save: bool,

    /// Display version information and exit
    #[arg(short = 'V', long)]
    pub version: bool,
}
