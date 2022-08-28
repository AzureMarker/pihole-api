// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Clients Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::{FtlClient, FtlMemory},
    routes::{
        auth::User,
        stats::common::{remove_excluded_clients, remove_hidden_clients},
    },
    services::PiholeModule,
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_result, Error, Reply},
};
use rocket::State;
use shaku_rocket::Inject;

pub use top_clients as route;

/// Get the top clients
#[get("/stats/top_clients?<params..>")]
pub fn top_clients(
    _auth: User,
    ftl_memory: &State<FtlMemory>,
    env: Inject<PiholeModule, Env>,
    params: TopClientParams,
) -> Reply {
    reply_result(get_top_clients(ftl_memory, &env, params))
}

/// Represents the possible GET parameters on `/stats/top_clients`
#[derive(FromForm, Default)]
pub struct TopClientParams {
    pub limit: Option<usize>,
    pub inactive: Option<bool>,
    pub ascending: Option<bool>,
    pub blocked: Option<bool>,
}

/// Represents the reply structure for top (blocked) clients
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct TopClientsReply {
    pub top_clients: Vec<TopClientItemReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_queries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_queries: Option<usize>,
}

/// Represents the reply structure for a top (blocked) client item
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct TopClientItemReply {
    pub name: String,
    pub ip: String,
    pub count: usize,
}

/// Get the top clients according to the parameters
fn get_top_clients(
    ftl_memory: &FtlMemory,
    env: &Env,
    params: TopClientParams,
) -> Result<TopClientsReply, Error> {
    // Resolve the parameters
    let limit = params.limit.unwrap_or(10);
    let inactive = params.inactive.unwrap_or(false);
    let ascending = params.ascending.unwrap_or(false);
    let blocked = params.blocked.unwrap_or(false);

    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

    let total_count = if blocked {
        counters.blocked_queries
    } else {
        counters.total_queries
    } as usize;

    // Check if the client details are private
    if let Some(reply) = check_privacy_level_top_clients(env, blocked, total_count)? {
        // We can not share any of the clients, so use the reply returned by the
        // function
        return Ok(reply);
    }

    let strings = ftl_memory.strings(&lock)?;
    let clients = ftl_memory.clients(&lock)?;

    // Get an array of valid client references (FTL allocates more than it uses)
    let mut clients: Vec<&FtlClient> = clients
        .iter()
        .take(counters.total_clients as usize)
        .collect();

    // Ignore inactive clients by default (retain active clients)
    if !inactive {
        if blocked {
            clients.retain(|client| client.blocked_count > 0);
        } else {
            clients.retain(|client| client.query_count > 0);
        }
    }

    // Remove excluded and hidden clients
    remove_excluded_clients(&mut clients, env, &strings)?;
    remove_hidden_clients(&mut clients, &strings);

    // Sort the clients (descending by default)
    match (ascending, blocked) {
        (false, false) => clients.sort_by(|a, b| b.query_count.cmp(&a.query_count)),
        (true, false) => clients.sort_by(|a, b| a.query_count.cmp(&b.query_count)),
        (false, true) => clients.sort_by(|a, b| b.blocked_count.cmp(&a.blocked_count)),
        (true, true) => clients.sort_by(|a, b| a.blocked_count.cmp(&b.blocked_count)),
    }

    // Take into account the limit
    if limit < clients.len() {
        clients.truncate(limit);
    }

    // Map the clients into the output format
    let top_clients: Vec<TopClientItemReply> = clients
        .into_iter()
        .map(|client| {
            let name = client.get_name(&strings).unwrap_or_default().to_owned();
            let ip = client.get_ip(&strings).to_owned();
            let count = if blocked {
                client.blocked_count
            } else {
                client.query_count
            } as usize;

            TopClientItemReply { name, ip, count }
        })
        .collect();

    // Output format changes when getting top blocked clients
    if blocked {
        Ok(TopClientsReply {
            top_clients,
            total_queries: None,
            blocked_queries: Some(counters.blocked_queries as usize),
        })
    } else {
        Ok(TopClientsReply {
            top_clients,
            total_queries: Some(counters.total_queries as usize),
            blocked_queries: None,
        })
    }
}

/// Check the privacy level to see if clients are allowed to be shared. If not,
/// then only return the relevant count (total or blocked queries).
pub fn check_privacy_level_top_clients(
    env: &Env,
    blocked: bool,
    count: usize,
) -> Result<Option<TopClientsReply>, Error> {
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(env)?
        >= FtlPrivacyLevel::HideDomainsAndClients
    {
        return if blocked {
            Ok(Some(TopClientsReply {
                top_clients: Vec::new(),
                total_queries: None,
                blocked_queries: Some(count),
            }))
        } else {
            Ok(Some(TopClientsReply {
                top_clients: Vec::new(),
                total_queries: Some(count),
                blocked_queries: None,
            }))
        };
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{FtlClient, FtlCounters, FtlMemory, FtlSettings},
        testing::TestBuilder,
    };
    use std::collections::HashMap;

    /// There are 6 clients, two inactive, one hidden, and two with names.
    fn test_data() -> FtlMemory {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "10.1.1.3".to_owned());
        strings.insert(5, "client3".to_owned());
        strings.insert(6, "10.1.1.4".to_owned());
        strings.insert(7, "10.1.1.5".to_owned());
        strings.insert(8, "0.0.0.0".to_owned());

        FtlMemory::Test {
            clients: vec![
                FtlClient::new(30, 10, 1, Some(2)),
                FtlClient::new(20, 5, 3, None),
                FtlClient::new(10, 0, 4, Some(5)),
                FtlClient::new(40, 0, 6, None),
                FtlClient::new(0, 0, 7, None),
                FtlClient::new(0, 0, 8, None),
            ],
            domains: Vec::new(),
            over_time: Vec::new(),
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters {
                total_queries: 100,
                blocked_queries: 15,
                total_clients: 6,
                ..FtlCounters::default()
            },
            settings: FtlSettings::default(),
        }
    }

    /// The default behavior lists all active clients in descending order
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Show only active blocked clients (active in terms of blocked query
    /// count)
    #[test]
    fn blocked_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?blocked=true")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "client1", "ip": "10.1.1.1", "count": 10 },
                    { "name": "",        "ip": "10.1.1.2", "count": 5 }
                ],
                "blocked_queries": 15
            }))
            .test();
    }

    /// The number of clients shown is <= the limit
    #[test]
    fn limit() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?limit=2")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Same as the default behavior but in ascending order
    #[test]
    fn ascending() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?ascending=true")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.4", "count": 40 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Privacy level 2 does not show any clients
    #[test]
    fn privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!({
                "top_clients": [],
                "total_queries": 100
            }))
            .test();
    }

    /// Privacy level 2 does not show any clients, and has a
    /// `"blocked_queries`" key instead of a `"total_queries"` key
    #[test]
    fn privacy_blocked() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?blocked=true")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!({
                "top_clients": [],
                "blocked_queries": 15
            }))
            .test();
    }

    /// Inactive clients are shown, but hidden clients are still not shown
    #[test]
    fn inactive_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?inactive=true")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 },
                    { "name": "",        "ip": "10.1.1.5", "count":  0 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Excluded clients are not shown
    #[test]
    fn excluded_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(
                PiholeFile::SetupVars,
                "API_EXCLUDE_CLIENTS=client3,10.1.1.2",
            )
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 }
                ],
                "total_queries": 100
            }))
            .test();
    }
}
