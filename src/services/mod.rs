// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Services (and supporting code) of the API
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

pub mod domain_audit;
pub mod lists;

use crate::{
    databases::{
        ftl::{FtlDatabase, FtlDatabasePool},
        gravity::{GravityDatabase, GravityDatabasePool},
    },
    env::Env,
    ftl::FtlConnectionType,
};
use domain_audit::DomainAuditRepositoryImpl;
use lists::{ListRepositoryImpl, ListServiceImpl};
use shaku::module;

module! {
    pub PiholeModule {
        components = [
            Env,
            FtlConnectionType,
            GravityDatabasePool,
            FtlDatabasePool
        ],
        providers = [
            ListRepositoryImpl,
            ListServiceImpl,
            DomainAuditRepositoryImpl,
            GravityDatabase,
            FtlDatabase
        ]
    }
}
