// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Query Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQueryType, settings::FtlPrivacyLevel};
use rocket::{
    form,
    form::{FromFormField, ValueField}
};

/// A list of query statuses which mark a query as blocked
pub const BLOCKED_STATUSES: [i32; 6] = [
    FtlQueryStatus::Gravity as i32,
    FtlQueryStatus::Wildcard as i32,
    FtlQueryStatus::Blacklist as i32,
    FtlQueryStatus::ExternalBlockIp as i32,
    FtlQueryStatus::ExternalBlockNull as i32,
    FtlQueryStatus::ExternalBlockNxdomainRa as i32
];

/// The query struct stored in shared memory
#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Copy, Clone)]
pub struct FtlQuery {
    pub magic: libc::c_uchar,
    pub query_type: FtlQueryType,
    pub status: FtlQueryStatus,
    pub privacy_level: FtlPrivacyLevel,
    pub reply_type: FtlQueryReplyType,
    pub dnssec_type: FtlDnssecType,
    pub timestamp: libc::time_t,
    pub domain_id: libc::c_int,
    pub client_id: libc::c_int,
    pub upstream_id: libc::c_int,
    pub id: libc::c_int,
    /// Saved in units of 1/10 milliseconds (1 = 0.1ms, 2 = 0.2ms,
    /// 2500 = 250.0ms, etc.)
    pub response_time: libc::c_ulong,
    pub database_id: i64,
    pub time_index: libc::c_uint,
    pub is_complete: bool
}

impl FtlQuery {
    /// Check if the query was blocked
    pub fn is_blocked(&self) -> bool {
        BLOCKED_STATUSES.contains(&(self.status as i32))
    }
}

/// The statuses an FTL query can have
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
pub enum FtlQueryStatus {
    Unknown,
    Gravity,
    Forward,
    Cache,
    Wildcard,
    Blacklist,
    ExternalBlockIp,
    ExternalBlockNull,
    ExternalBlockNxdomainRa
}

impl FtlQueryStatus {
    /// Get the query status from its ordinal value
    pub fn from_number(num: isize) -> Option<Self> {
        match num {
            0 => Some(FtlQueryStatus::Unknown),
            1 => Some(FtlQueryStatus::Gravity),
            2 => Some(FtlQueryStatus::Forward),
            3 => Some(FtlQueryStatus::Cache),
            4 => Some(FtlQueryStatus::Wildcard),
            5 => Some(FtlQueryStatus::Blacklist),
            6 => Some(FtlQueryStatus::ExternalBlockIp),
            7 => Some(FtlQueryStatus::ExternalBlockNull),
            8 => Some(FtlQueryStatus::ExternalBlockNxdomainRa),
            _ => None
        }
    }
}

impl<'v> FromFormField<'v> for FtlQueryStatus {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        let num = field
            .value
            .parse::<u8>()
            .map_err(|_| form::Error::validation("Not a number"))?;
        Self::from_number(num as isize)
            .ok_or_else(|| form::Error::validation("Unknown FTL query status").into())
    }
}

/// The reply types an FTL query can have
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
pub enum FtlQueryReplyType {
    Unknown,
    NODATA,
    NXDOMAIN,
    CNAME,
    IP,
    DOMAIN,
    RRNAME,
    SERVFAIL,
    REFUSED,
    NOTIMP,
    OTHER
}

impl FtlQueryReplyType {
    /// Get the query reply type from its ordinal value
    pub fn from_number(num: isize) -> Option<Self> {
        match num {
            0 => Some(FtlQueryReplyType::Unknown),
            1 => Some(FtlQueryReplyType::NODATA),
            2 => Some(FtlQueryReplyType::NXDOMAIN),
            3 => Some(FtlQueryReplyType::CNAME),
            4 => Some(FtlQueryReplyType::IP),
            5 => Some(FtlQueryReplyType::DOMAIN),
            6 => Some(FtlQueryReplyType::RRNAME),
            7 => Some(FtlQueryReplyType::SERVFAIL),
            8 => Some(FtlQueryReplyType::REFUSED),
            9 => Some(FtlQueryReplyType::NOTIMP),
            10 => Some(FtlQueryReplyType::OTHER),
            _ => None
        }
    }
}

impl<'v> FromFormField<'v> for FtlQueryReplyType {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        let num = field
            .value
            .parse::<u8>()
            .map_err(|_| form::Error::validation("Not a number"))?;
        Self::from_number(num as isize)
            .ok_or_else(|| form::Error::validation("Unknown FTL query reply type").into())
    }
}

/// The DNSSEC reply types an FTL query can have
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
pub enum FtlDnssecType {
    Unspecified,
    Secure,
    Insecure,
    Bogus,
    Abandoned,
    Unknown
}

impl FtlDnssecType {
    /// Get the DNSSEC type from its ordinal value
    pub fn from_number(num: isize) -> Option<Self> {
        match num {
            0 => Some(FtlDnssecType::Unspecified),
            1 => Some(FtlDnssecType::Secure),
            2 => Some(FtlDnssecType::Insecure),
            3 => Some(FtlDnssecType::Bogus),
            4 => Some(FtlDnssecType::Abandoned),
            5 => Some(FtlDnssecType::Unknown),
            _ => None
        }
    }
}

impl<'v> FromFormField<'v> for FtlDnssecType {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        let num = field
            .value
            .parse::<u8>()
            .map_err(|_| form::Error::validation("Not a number"))?;
        Self::from_number(num as isize)
            .ok_or_else(|| form::Error::validation("Unknown FTL DNSSEC type").into())
    }
}
