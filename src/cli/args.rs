// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// CLI Arguments and Options
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{cli::handler::get_version, env::DEFAULT_CONFIG_LOCATION};
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};

/// This defines the arguments that the CLI can be given.
/// `AppSettings::VersionlessSubcommands` will remove the `-V` version flag from
/// sub-commands. All sub-commands in this project have the same version.
#[derive(StructOpt)]
#[structopt(
    name = "pihole-API",
    about = "An HTTP API for Pi-hole.",
    version = get_version(),
    global_setting = AppSettings::VersionlessSubcommands
)]
pub struct CliArgs {
    #[structopt(subcommand)]
    pub command: Option<CliCommand>,

    /// The location of the config
    #[structopt(default_value = DEFAULT_CONFIG_LOCATION, long)]
    pub config: PathBuf,
}

// The commands that the CLI handles
#[derive(StructOpt)]
pub enum CliCommand {
    /// Prints version information
    #[structopt(version = get_version())]
    Version,
    /// Prints branch
    #[structopt(version = get_version())]
    Branch,
    /// Prints git hash
    #[structopt(version = get_version())]
    Hash,
    /// Generate the dns server configuration
    #[structopt(version = get_version())]
    GenerateDnsConfig,
}
