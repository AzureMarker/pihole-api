// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Domain Audit Log Database Repository
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::gravity::GravityDatabase,
    util::{Error, ErrorKind}
};
use diesel::{expression::exists::exists, insert_into, prelude::*, select};
use failure::ResultExt;
use shaku::{ProvidedInterface, Provider};

/// Describes interactions with the domain audit data store
#[cfg_attr(test, mockall::automock)]
pub trait DomainAuditRepository: ProvidedInterface {
    /// Check if the domain is contained in the audit table
    fn contains(&self, domain: &str) -> Result<bool, Error>;

    /// Get all audited domains
    fn get_all(&self) -> Result<Vec<String>, Error>;

    /// Add a domain to the audit table
    fn add(&self, domain: &str) -> Result<(), Error>;
}

/// The implementation of `DomainAuditRepository`
#[derive(Provider)]
#[shaku(interface = DomainAuditRepository)]
pub struct DomainAuditRepositoryImpl {
    #[shaku(provide)]
    db: Box<GravityDatabase>
}

impl DomainAuditRepository for DomainAuditRepositoryImpl {
    fn contains(&self, input_domain: &str) -> Result<bool, Error> {
        use crate::databases::gravity::domain_audit::dsl::*;
        let db = &self.db as &SqliteConnection;

        select(exists(domain_audit.filter(domain.eq(input_domain))))
            .get_result(db)
            .context(ErrorKind::GravityDatabase)
            .map_err(Error::from)
    }

    fn get_all(&self) -> Result<Vec<String>, Error> {
        use crate::databases::gravity::domain_audit::dsl::*;
        let db = &self.db as &SqliteConnection;

        domain_audit
            .select(domain)
            .load(db)
            .context(ErrorKind::GravityDatabase)
            .map_err(Error::from)
    }

    fn add(&self, input_domain: &str) -> Result<(), Error> {
        use crate::databases::gravity::domain_audit::dsl::*;
        let db = &self.db as &SqliteConnection;

        insert_into(domain_audit)
            .values(domain.eq(input_domain))
            .execute(db)
            .context(ErrorKind::GravityDatabase)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        databases::gravity::connect_to_gravity_test_db,
        services::domain_audit::{DomainAuditRepository, DomainAuditRepositoryImpl}
    };

    /// If the audit table contains the domain, true will be returned
    #[test]
    fn contains_true() {
        let db = connect_to_gravity_test_db();
        let repo = DomainAuditRepositoryImpl { db };

        assert_eq!(repo.contains("audited.domain").unwrap(), true);
    }

    /// If the audit table does not contain the domain, false will be returned
    #[test]
    fn contains_false() {
        let db = connect_to_gravity_test_db();
        let repo = DomainAuditRepositoryImpl { db };

        assert_eq!(repo.contains("not.audited.domain").unwrap(), false);
    }

    /// All audited domains are retrieved
    #[test]
    fn get_all() {
        let db = connect_to_gravity_test_db();
        let repo = DomainAuditRepositoryImpl { db };

        assert_eq!(repo.get_all().unwrap(), vec!["audited.domain".to_owned()]);
    }

    /// After adding, the database will contain the domain
    #[test]
    fn add_success() {
        let db = connect_to_gravity_test_db();
        let repo = DomainAuditRepositoryImpl { db };

        repo.add("new.audited.domain").unwrap();

        assert_eq!(repo.contains("new.audited.domain").unwrap(), true);
    }
}
