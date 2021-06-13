// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// List Service (Whitelist, Blacklist, Regexlist)
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlConnectionType,
    services::lists::{List, ListRepository},
    util::{Error, ErrorKind},
};
use failure::ResultExt;
use shaku::Provider;
use std::{
    process::{Command, Stdio},
    sync::Arc,
};

/// Describes interactions with the Pi-hole domain lists (whitelist, blacklist,
/// and regexlist)
#[cfg_attr(test, mockall::automock)]
pub trait ListService: Send {
    /// Add a domain to the list and update FTL and other lists accordingly.
    /// Example: when adding to the whitelist, remove from the blacklist.
    fn add(&self, list: List, domain: &str) -> Result<(), Error>;

    /// Remove a domain from the list and update FTL
    fn remove(&self, list: List, domain: &str) -> Result<(), Error>;

    /// Get all of the domains in the list
    fn get(&self, list: List) -> Result<Vec<String>, Error>;
}

/// The implementation of `ListService`
#[derive(Provider)]
#[shaku(interface = ListService)]
pub struct ListServiceImpl {
    #[shaku(provide)]
    repo: Box<dyn ListRepository>,
    #[shaku(inject)]
    env: Arc<Env>,
    #[shaku(inject)]
    ftl: Arc<FtlConnectionType>,
}

impl ListService for ListServiceImpl {
    fn add(&self, list: List, domain: &str) -> Result<(), Error> {
        match list {
            List::White => {
                // We need to add it to the whitelist and remove it from the
                // blacklist
                self.add_raw(List::White, domain)?;
                self.try_remove_raw(List::Black, domain)?;

                // Since we haven't hit an error yet, reload gravity
                reload_gravity(List::White, &self.env)
            }
            List::Black => {
                // We need to add it to the blacklist and remove it from the
                // whitelist
                self.add_raw(List::Black, domain)?;
                self.try_remove_raw(List::White, domain)?;

                // Since we haven't hit an error yet, reload gravity
                reload_gravity(List::Black, &self.env)
            }
            List::Regex => {
                // We only need to add it to the regex list
                self.add_raw(List::Regex, domain)?;

                // Since we haven't hit an error yet, tell FTL to recompile
                // regex
                self.ftl.connect("recompile-regex")?.expect_eom()
            }
        }
    }

    fn remove(&self, list: List, domain: &str) -> Result<(), Error> {
        match list {
            List::White => {
                self.remove_raw(List::White, domain)?;
                reload_gravity(List::White, &self.env)
            }
            List::Black => {
                self.remove_raw(List::Black, domain)?;
                reload_gravity(List::Black, &self.env)
            }
            List::Regex => {
                self.remove_raw(List::Regex, domain)?;
                self.ftl.connect("recompile-regex")?.expect_eom()
            }
        }
    }

    fn get(&self, list: List) -> Result<Vec<String>, Error> {
        self.repo.get(list)
    }
}

impl ListServiceImpl {
    /// Simply add a domain to the list
    fn add_raw(&self, list: List, domain: &str) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !list.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is already in the list
        if self.repo.contains(list, domain)? {
            return Err(Error::from(ErrorKind::AlreadyExists));
        }

        self.repo.add(list, domain)
    }

    /// Try to remove a domain from the list, but it is not an error if the
    /// domain does not exist
    fn try_remove_raw(&self, list: List, domain: &str) -> Result<(), Error> {
        match self.remove_raw(list, domain) {
            // Pass through successful results
            Ok(_) => Ok(()),
            Err(e) => {
                // Ignore NotFound errors
                if e.kind() == ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Simply remove a domain from the list
    fn remove_raw(&self, list: List, domain: &str) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !list.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is not in the list
        if !self.repo.contains(list, domain)? {
            return Err(Error::from(ErrorKind::NotFound));
        }

        self.repo.remove(list, domain)
    }
}

/// Reload Gravity to activate changes in lists
pub fn reload_gravity(list: List, env: &Env) -> Result<(), Error> {
    // Don't actually reload Gravity during testing
    if env.is_test() {
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("pihole")
        .arg("-g")
        .arg("--skip-download")
        // Based on what list we modified, only reload what is necessary
        .arg(match list {
            List::White => "--whitelist-only",
            List::Black => "--blacklist-only",
            _ => return Err(Error::from(ErrorKind::Unknown))
        })
        // Ignore stdin, stdout, and stderr
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Get the returned status code
        .status()
        .context(ErrorKind::GravityError)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(ErrorKind::GravityError))
    }
}

#[cfg(test)]
mod test {
    use super::List;
    use crate::{
        ftl::FtlConnectionType,
        services::lists::{ListService, ListServiceImpl, MockListRepository},
        testing::{write_eom, TestEnvBuilder},
    };
    use mockall::predicate::*;
    use std::{collections::HashMap, sync::Arc};

    fn get_ftl() -> FtlConnectionType {
        let mut data = Vec::new();
        let mut command_map = HashMap::new();

        write_eom(&mut data);
        command_map.insert("recompile-regex".to_owned(), data);

        FtlConnectionType::Test(command_map)
    }

    /// Test getting the domains for a list
    fn get_test(list: List, domain: &str) {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let mut repo = MockListRepository::new();

        repo.expect_get()
            .with(eq(list))
            .return_const(Ok(vec![domain.to_owned()]));

        let service = ListServiceImpl {
            repo: Box::new(repo),
            env: Arc::new(env),
            ftl: Arc::new(ftl),
        };

        assert_eq!(service.get(list).unwrap(), vec![domain.to_owned()]);
    }

    /// Test successfully deleting a domain from a list
    fn delete_test(list: List, domain: &'static str) {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let mut repo = MockListRepository::new();

        repo.expect_contains()
            .with(eq(list), eq(domain))
            .return_const(Ok(true));
        repo.expect_remove()
            .with(eq(list), eq(domain))
            .return_const(Ok(()));

        let service = ListServiceImpl {
            repo: Box::new(repo),
            env: Arc::new(env),
            ftl: Arc::new(ftl),
        };

        service.remove(list, domain).unwrap();
    }

    /// The lists are retrieved correctly
    #[test]
    fn get() {
        get_test(List::White, "whitelist.com");
        get_test(List::Black, "blacklist.com");
        get_test(List::Regex, "regex.com");
    }

    /// Adding a domain to the whitelist works when the domain does not exist
    /// in either the whitelist or blacklist
    #[test]
    fn add_whitelist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let mut repo = MockListRepository::new();

        repo.expect_contains()
            .with(eq(List::White), eq("example.com"))
            .return_const(Ok(false));
        repo.expect_add()
            .with(eq(List::White), eq("example.com"))
            .return_const(Ok(()));
        repo.expect_contains()
            .with(eq(List::Black), eq("example.com"))
            .return_const(Ok(false));

        let service = ListServiceImpl {
            repo: Box::new(repo),
            env: Arc::new(env),
            ftl: Arc::new(ftl),
        };

        service.add(List::White, "example.com").unwrap();
    }

    /// Adding a domain to the blacklist works when the domain does not exist
    /// in either the whitelist or blacklist
    #[test]
    fn add_blacklist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let mut repo = MockListRepository::new();

        repo.expect_contains()
            .with(eq(List::Black), eq("example.com"))
            .return_const(Ok(false));
        repo.expect_add()
            .with(eq(List::Black), eq("example.com"))
            .return_const(Ok(()));
        repo.expect_contains()
            .with(eq(List::White), eq("example.com"))
            .return_const(Ok(false));

        let service = ListServiceImpl {
            repo: Box::new(repo),
            env: Arc::new(env),
            ftl: Arc::new(ftl),
        };

        service.add(List::Black, "example.com").unwrap();
    }

    /// Adding a domain to the regex list works when the domain does not already
    /// exist in the regex list
    #[test]
    fn add_regexlist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let mut repo = MockListRepository::new();

        repo.expect_contains()
            .with(eq(List::Regex), eq("example.com"))
            .return_const(Ok(false));
        repo.expect_add()
            .with(eq(List::Regex), eq("example.com"))
            .return_const(Ok(()));

        let service = ListServiceImpl {
            repo: Box::new(repo),
            env: Arc::new(env),
            ftl: Arc::new(ftl),
        };

        service.add(List::Regex, "example.com").unwrap();
    }

    #[test]
    fn delete_whitelist() {
        delete_test(List::White, "whitelist.com");
    }

    #[test]
    fn delete_blacklist() {
        delete_test(List::Black, "blacklist.com");
    }

    #[test]
    fn delete_regexlist() {
        delete_test(List::Regex, "regex.com");
    }
}
