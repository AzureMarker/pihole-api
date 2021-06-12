// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Statistic API Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

pub mod clients;
pub mod common;
pub mod database;
pub mod history;
pub mod over_time_clients;
pub mod over_time_history;
pub mod query_types;
pub mod recent_blocked;
pub mod summary;
pub mod top_clients;
pub mod top_domains;
pub mod upstreams;
