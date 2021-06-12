// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Settings - Database Statistics
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::FtlConnectionType,
    routes::auth::User,
    services::PiholeModule,
    util::{reply_data, Reply}
};
use shaku_rocket::Inject;

/// Read db stats from FTL
#[get("/settings/ftldb")]
pub fn get_ftldb(ftl: Inject<PiholeModule, FtlConnectionType>, _auth: User) -> Reply {
    let mut con = ftl.connect("dbstats")?;

    // Read in FTL's database stats
    let db_queries = con.read_i32()?;
    let db_filesize = con.read_i64()?;
    let mut version_buffer = [0u8; 64];
    let db_sqlite_version = con.read_str(&mut version_buffer)?;
    con.expect_eom()?;

    reply_data(json!({
        "queries": db_queries,
        "filesize": db_filesize,
        "sqlite_version": db_sqlite_version
    }))
}

#[cfg(test)]
mod test {
    use crate::testing::{write_eom, TestBuilder};
    use rmp::encode;

    /// Basic test for reported values
    #[test]
    fn test_get_ftldb() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1_048_576).unwrap();
        encode::write_i64(&mut data, 32768).unwrap();
        encode::write_str(&mut data, "3.0.1").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/settings/ftldb")
            .ftl("dbstats", data)
            .expect_json(json!({
                "queries": 1_048_576,
                "filesize": 32768,
                "sqlite_version": "3.0.1"
            }))
            .test();
    }
}
