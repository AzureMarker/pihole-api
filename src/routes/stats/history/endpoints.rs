// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    env::Env,
    ftl::{FtlDnssecType, FtlMemory, FtlQueryReplyType, FtlQueryStatus, FtlQueryType},
    routes::{auth::User, stats::history::get_history::get_history},
    services::PiholeModule,
    util::{reply_result, Error, ErrorKind, Reply}
};
use base64::{decode, encode};
use failure::ResultExt;
use rocket::{
    http::RawStr,
    request::{Form, FromFormValue},
    State
};
use shaku_rocket::{Inject, InjectProvided};

/// Get the query history according to the specified parameters
#[get("/stats/history?<params..>")]
pub fn history(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: Inject<PiholeModule, Env>,
    params: Form<HistoryParams>,
    db: InjectProvided<PiholeModule, FtlDatabase>
) -> Reply {
    reply_result(get_history(&ftl_memory, &env, params.into_inner(), &db))
}

/// The structure returned by the history endpoint
#[derive(Serialize, PartialEq, Debug)]
pub struct HistoryReply {
    pub history: Vec<QueryReply>,
    pub cursor: Option<String>
}

/// The structure of queries returned by the history endpoint
#[derive(Serialize, PartialEq, Debug)]
pub struct QueryReply {
    pub timestamp: u64,
    pub r#type: u8,
    pub status: u8,
    pub domain: String,
    pub client: String,
    pub dnssec: u8,
    pub reply: u8,
    pub response_time: u32
}

/// Represents the possible GET parameters on `/stats/history`
#[derive(FromForm)]
pub struct HistoryParams {
    pub cursor: Option<HistoryCursor>,
    pub from: Option<u64>,
    pub until: Option<u64>,
    pub domain: Option<String>,
    pub client: Option<String>,
    pub upstream: Option<String>,
    pub query_type: Option<FtlQueryType>,
    pub status: Option<FtlQueryStatus>,
    pub blocked: Option<bool>,
    pub dnssec: Option<FtlDnssecType>,
    pub reply: Option<FtlQueryReplyType>,
    pub limit: Option<usize>
}

impl Default for HistoryParams {
    fn default() -> Self {
        HistoryParams {
            cursor: None,
            from: None,
            until: None,
            domain: None,
            client: None,
            upstream: None,
            query_type: None,
            status: None,
            blocked: None,
            dnssec: None,
            reply: None,
            limit: Some(100)
        }
    }
}

/// The cursor object used for history pagination
#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct HistoryCursor {
    pub id: Option<i32>,
    pub db_id: Option<i64>
}

impl HistoryCursor {
    /// Get the Base64 representation of the cursor
    pub fn as_base64(&self) -> Result<String, Error> {
        let bytes = serde_json::to_vec(self).context(ErrorKind::Unknown)?;

        Ok(encode(&bytes))
    }
}

impl<'a> FromFormValue<'a> for HistoryCursor {
    type Error = Error;

    fn from_form_value(form_value: &'a RawStr) -> Result<Self, Self::Error> {
        // Decode from Base64
        let decoded = decode(form_value).context(ErrorKind::BadRequest)?;

        // Deserialize from JSON
        let cursor = serde_json::from_slice(&decoded).context(ErrorKind::BadRequest)?;

        Ok(cursor)
    }
}
