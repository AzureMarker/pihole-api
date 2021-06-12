// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Gravity Database Models
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::{
        common::DatabaseService,
        custom_connection::{
            CustomDBConfig, CustomSqliteConnection, CustomSqliteConnectionManager
        }
    },
    settings::{ConfigEntry, FtlConfEntry},
    util::{self, ErrorKind}
};
use diesel::{r2d2::Pool, SqliteConnection};
use failure::{Fail, ResultExt};
use rocket_sync_db_pools::r2d2::PooledConnection;
use shaku::{Component, HasComponent, Module, Provider};
use std::{error::Error, ops::Deref};

fn default_connection() -> Pool<CustomSqliteConnectionManager> {
    let config = CustomDBConfig {
        url: FtlConfEntry::GravityDb.get_default().to_owned(),
        pool_size: 8,
        test_schema: None
    };

    CustomSqliteConnection::pool(config).unwrap()
}

#[derive(Component)]
#[shaku(interface = DatabaseService<GravityDatabase>)]
pub struct GravityDatabasePool {
    #[shaku(default = default_connection())]
    pool: Pool<CustomSqliteConnectionManager>
}

impl DatabaseService<GravityDatabase> for GravityDatabasePool {
    fn get_connection(&self) -> Result<GravityDatabase, util::Error> {
        self.pool
            .get()
            .map(GravityDatabase)
            .context(ErrorKind::GravityDatabase)
            .map_err(Into::into)
    }
}

pub struct GravityDatabase(pub PooledConnection<CustomSqliteConnectionManager>);

impl Deref for GravityDatabase {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M: Module + HasComponent<dyn DatabaseService<GravityDatabase>>> Provider<M>
    for GravityDatabase
{
    type Interface = Self;

    fn provide(module: &M) -> Result<Box<Self::Interface>, Box<dyn Error + 'static>> {
        let pool: &dyn DatabaseService<GravityDatabase> = module.resolve_ref();

        Ok(Box::new(pool.get_connection().map_err(Fail::compat)?))
    }
}
