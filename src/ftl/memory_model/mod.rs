// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Shared Memory Data Types
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

/// Used by FTL to check memory integrity in various structs
#[cfg(test)]
pub const MAGIC_BYTE: libc::c_uchar = 0x57;

mod client;
mod counters;
mod domain;
mod lock;
mod over_time;
mod query;
mod settings;
mod strings;
mod upstream;

pub use self::{
    client::*,
    counters::{FtlCounters, FtlQueryType},
    domain::{FtlDomain, FtlRegexMatch},
    lock::FtlLock,
    over_time::*,
    query::{FtlDnssecType, FtlQuery, FtlQueryReplyType, FtlQueryStatus, BLOCKED_STATUSES},
    settings::FtlSettings,
    strings::FtlStrings,
    upstream::FtlUpstream
};
