// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Removing Domains From Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    routes::auth::User,
    services::{
        lists::{List, ListService},
        PiholeModule
    },
    util::{reply_success, Reply}
};
use shaku_rocket::InjectProvided;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(
    _auth: User,
    list_service: InjectProvided<PiholeModule, dyn ListService>,
    domain: String
) -> Reply {
    list_service.remove(List::White, &domain)?;
    reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(
    _auth: User,
    list_service: InjectProvided<PiholeModule, dyn ListService>,
    domain: String
) -> Reply {
    list_service.remove(List::Black, &domain)?;
    reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(
    _auth: User,
    list_service: InjectProvided<PiholeModule, dyn ListService>,
    domain: String
) -> Reply {
    list_service.remove(List::Regex, &domain)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{
        services::lists::{List, ListService, MockListService},
        testing::TestBuilder
    };
    use mockall::predicate::*;
    use rocket::http::Method;

    /// Test that a successful delete returns success
    fn delete_test(list: List, endpoint: &str, domain: &'static str) {
        TestBuilder::new()
            .endpoint(endpoint)
            .method(Method::Delete)
            .mock_provider::<dyn ListService>(Box::new(move |_| {
                let mut service = MockListService::new();

                service
                    .expect_remove()
                    .with(eq(list), eq(domain))
                    .return_const(Ok(()));

                Ok(Box::new(service))
            }))
            .expect_json(json!({ "status": "success" }))
            .test();
    }

    #[test]
    fn test_delete_whitelist() {
        delete_test(
            List::White,
            "/admin/api/dns/whitelist/example.com",
            "example.com"
        );
    }

    #[test]
    fn test_delete_blacklist() {
        delete_test(
            List::Black,
            "/admin/api/dns/blacklist/example.com",
            "example.com"
        );
    }

    #[test]
    fn test_delete_regexlist() {
        delete_test(
            List::Regex,
            "/admin/api/dns/regexlist/%5E.%2Aexample.com%24",
            "^.*example.com$"
        );
    }
}
