// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Databases
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod common;
pub mod custom_connection;
pub mod ftl;
pub mod gravity;

#[cfg(test)]
pub use self::common::{create_memory_db, FakeDatabaseService};
pub use self::common::{load_ftl_db_config, load_gravity_db_config, DatabaseService};
