// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Reading Domain Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    services::{
        lists::{List, ListService},
        PiholeModule
    },
    util::{reply_result, Reply}
};
use shaku_rocket::InjectProvided;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(service: InjectProvided<PiholeModule, dyn ListService>) -> Reply {
    reply_result(service.get(List::White))
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(service: InjectProvided<PiholeModule, dyn ListService>) -> Reply {
    reply_result(service.get(List::Black))
}

/// Get the Regex list domains
#[get("/dns/regexlist")]
pub fn get_regexlist(service: InjectProvided<PiholeModule, dyn ListService>) -> Reply {
    reply_result(service.get(List::Regex))
}

#[cfg(test)]
mod test {
    use crate::{
        services::lists::{List, ListService, MockListService},
        testing::TestBuilder
    };
    use mockall::predicate::*;

    /// Test that the domains are returned correctly
    fn get_test(list: List, endpoint: &str, domains: Vec<String>) {
        TestBuilder::new()
            .endpoint(endpoint)
            .expect_json(json!(domains))
            .mock_provider::<dyn ListService>(Box::new(move |_| {
                let mut service = MockListService::new();

                service
                    .expect_get()
                    .with(eq(list))
                    .return_const(Ok(domains.clone()));

                Ok(Box::new(service))
            }))
            .test();
    }

    #[test]
    fn test_get_whitelist() {
        get_test(
            List::White,
            "/admin/api/dns/whitelist",
            vec!["example.com".to_owned(), "example.net".to_owned()]
        );
    }

    #[test]
    fn test_get_blacklist() {
        get_test(
            List::Black,
            "/admin/api/dns/blacklist",
            vec!["example.com".to_owned(), "example.net".to_owned()]
        );
    }

    #[test]
    fn test_get_regexlist() {
        get_test(
            List::Regex,
            "/admin/api/dns/regexlist",
            vec!["^.*example.com$".to_owned(), "example.net".to_owned()]
        );
    }
}
