// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common database functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::custom_connection::CustomDBConfig,
    env::Env,
    settings::{ConfigEntry, FtlConfEntry},
    util::Error
};
use shaku::Interface;

#[cfg(test)]
use crate::databases::custom_connection::{CustomSqliteConnection, CustomSqliteConnectionManager};
#[cfg(test)]
use diesel::{
    connection::{Connection, TransactionManager},
    r2d2::Pool,
    SqliteConnection
};

pub trait DatabaseService<C>: Interface {
    fn get_connection(&self) -> Result<C, shaku::Error>;
}

pub fn load_gravity_db_config(env: &Env) -> Result<CustomDBConfig, Error> {
    Ok(CustomDBConfig {
        url: FtlConfEntry::GravityDb.read(env)?,
        pool_size: 8,
        test_schema: None
    })
}

pub fn load_ftl_db_config(env: &Env) -> Result<CustomDBConfig, Error> {
    Ok(CustomDBConfig {
        url: FtlConfEntry::DbFile.read(env)?,
        pool_size: 8,
        test_schema: None
    })
}

/// Start a test transaction so the database does not get modified. If a
/// transaction is already running, it is rolled back.
#[cfg(test)]
pub fn start_test_transaction(db: &SqliteConnection) {
    let transaction_manager: &dyn TransactionManager<SqliteConnection> = db.transaction_manager();
    let depth = transaction_manager.get_transaction_depth();

    if depth > 0 {
        transaction_manager.rollback_transaction(db).unwrap();
    }

    db.begin_test_transaction().unwrap();
}

/// Create an in-memory SQLite database with the given schema (SQL commands)
#[cfg(test)]
pub fn create_memory_db(schema: &str, pool_size: u32) -> Pool<CustomSqliteConnectionManager> {
    let config = CustomDBConfig {
        url: ":memory:".to_owned(),
        pool_size,
        test_schema: Some(schema.to_owned())
    };

    CustomSqliteConnection::pool(config).unwrap()
}

#[cfg(test)]
pub struct FakeDatabaseService;
#[cfg(test)]
impl<C> DatabaseService<C> for FakeDatabaseService {
    fn get_connection(&self) -> Result<C, shaku::Error> {
        Err(shaku::Error::ResolveError(
            "Databases are disabled".to_string()
        ))
    }
}
