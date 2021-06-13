// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Models
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::{
        custom_connection::{
            CustomDBConfig, CustomSqliteConnection, CustomSqliteConnectionManager,
        },
        DatabaseService,
    },
    ftl::{FtlDnssecType, FtlQueryReplyType},
    routes::stats::history::QueryReply,
    settings::{ConfigEntry, FtlConfEntry},
    util,
    util::ErrorKind,
};
use diesel::SqliteConnection;
use failure::{Fail, ResultExt};
use rocket_sync_db_pools::r2d2::{Pool, PooledConnection};
use shaku::{Component, HasComponent, Module, Provider};
use std::{error::Error, ops::Deref};

fn default_connection() -> Pool<CustomSqliteConnectionManager> {
    let config = CustomDBConfig {
        url: FtlConfEntry::DbFile.get_default().to_owned(),
        pool_size: 8,
        test_schema: None,
    };

    CustomSqliteConnection::pool(config).unwrap()
}

#[derive(Component)]
#[shaku(interface = DatabaseService<FtlDatabase>)]
pub struct FtlDatabasePool {
    #[shaku(default = default_connection())]
    pool: Pool<CustomSqliteConnectionManager>,
}

impl DatabaseService<FtlDatabase> for FtlDatabasePool {
    fn get_connection(&self) -> Result<FtlDatabase, util::Error> {
        self.pool
            .get()
            .map(FtlDatabase)
            .context(ErrorKind::FtlDatabase)
            .map_err(util::Error::from)
    }
}

pub struct FtlDatabase(pub PooledConnection<CustomSqliteConnectionManager>);

impl Deref for FtlDatabase {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M: Module + HasComponent<dyn DatabaseService<FtlDatabase>>> Provider<M> for FtlDatabase {
    type Interface = Self;

    fn provide(module: &M) -> Result<Box<Self::Interface>, Box<dyn Error + 'static>> {
        let pool: &dyn DatabaseService<FtlDatabase> = module.resolve_ref();

        Ok(Box::new(pool.get_connection().map_err(Fail::compat)?))
    }
}

#[allow(dead_code)]
pub enum FtlTableEntry {
    Version,
    LastTimestamp,
    FirstCounterTimestamp,
}

#[allow(dead_code)]
pub enum CounterTableEntry {
    TotalQueries,
    BlockedQueries,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Queryable)]
pub struct FtlDbQuery {
    pub id: i32,
    pub timestamp: i32,
    pub query_type: i32,
    pub status: i32,
    pub domain: String,
    pub client: String,
    pub upstream: Option<String>,
}

impl From<FtlDbQuery> for QueryReply {
    fn from(query: FtlDbQuery) -> QueryReply {
        QueryReply {
            timestamp: query.timestamp as u64,
            r#type: query.query_type as u8,
            status: query.status as u8,
            domain: query.domain,
            client: query.client,
            dnssec: FtlDnssecType::Unknown as u8,
            reply: FtlQueryReplyType::Unknown as u8,
            response_time: 0,
        }
    }
}
