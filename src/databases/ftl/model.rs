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
            CustomDBConfig, CustomSqliteConnection, CustomSqliteConnectionManager
        },
        DatabaseService
    },
    ftl::{FtlDnssecType, FtlQueryReplyType},
    routes::stats::QueryReply,
    settings::{ConfigEntry, FtlConfEntry}
};
use diesel::SqliteConnection;
use rocket_contrib::databases::r2d2::{Pool, PooledConnection};
use shaku::{Component, Container, HasComponent, Module, Provider};
use std::ops::Deref;

fn default_connection() -> Pool<CustomSqliteConnectionManager> {
    let config = CustomDBConfig {
        url: FtlConfEntry::DbFile.get_default().to_owned(),
        pool_size: 8,
        test_schema: None
    };

    CustomSqliteConnection::pool(config).unwrap()
}

#[derive(Component)]
#[shaku(interface = DatabaseService<FtlDatabase>)]
pub struct FtlDatabasePool {
    #[shaku(default = default_connection())]
    pool: Pool<CustomSqliteConnectionManager>
}

impl DatabaseService<FtlDatabase> for FtlDatabasePool {
    fn get_connection(&self) -> Result<FtlDatabase, shaku::Error> {
        self.pool
            .get()
            .map(FtlDatabase)
            .map_err(|e| shaku::Error::ResolveError(e.to_string()))
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

    fn provide(container: &Container<M>) -> Result<Box<Self::Interface>, shaku::Error> {
        let pool: &dyn DatabaseService<FtlDatabase> = container.resolve_ref();

        Ok(Box::new(pool.get_connection()?))
    }
}

#[allow(dead_code)]
pub enum FtlTableEntry {
    Version,
    LastTimestamp,
    FirstCounterTimestamp
}

#[allow(dead_code)]
pub enum CounterTableEntry {
    TotalQueries,
    BlockedQueries
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
    pub upstream: Option<String>
}

impl Into<QueryReply> for FtlDbQuery {
    fn into(self) -> QueryReply {
        QueryReply {
            timestamp: self.timestamp as u64,
            r#type: self.query_type as u8,
            status: self.status as u8,
            domain: self.domain,
            client: self.client,
            dnssec: FtlDnssecType::Unknown as u8,
            reply: FtlQueryReplyType::Unknown as u8,
            response_time: 0
        }
    }
}
