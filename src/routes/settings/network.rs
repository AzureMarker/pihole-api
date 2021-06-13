// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Local Network Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    routes::auth::User,
    services::PiholeModule,
    settings::{ConfigEntry, SetupVarsEntry},
    util::{reply_data, Reply},
};
use shaku_rocket::Inject;
use std::ffi::OsString;

/// Get Pi-hole local network information
#[get("/settings/network")]
pub fn get_network(env: Inject<PiholeModule, Env>, _auth: User) -> Reply {
    let ipv4_full = SetupVarsEntry::Ipv4Address.read(&env)?;
    let ipv4_address: Vec<&str> = ipv4_full.split('/').collect();
    let ipv6_full = SetupVarsEntry::Ipv6Address.read(&env)?;
    let ipv6_address: Vec<&str> = ipv6_full.split('/').collect();

    reply_data(json!({
        "interface": SetupVarsEntry::PiholeInterface.read(&env)?,
        "ipv4_address": ipv4_address[0],
        "ipv6_address": ipv6_address[0],
        "hostname": hostname::get().unwrap_or_else(|_| OsString::from("unknown"))
    }))
}

#[cfg(test)]
mod test {
    use crate::{env::PiholeFile, testing::TestBuilder};
    use std::ffi::OsString;

    /// Basic test for reported settings
    #[test]
    fn test_get_network() {
        let current_host = hostname::get().unwrap_or_else(|_| OsString::from("unknown"));

        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(
                PiholeFile::SetupVars,
                "IPV4_ADDRESS=192.168.1.205/24\n\
                 IPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\n\
                 PIHOLE_INTERFACE=eth0\n",
            )
            .expect_json(json!({
                "interface": "eth0",
                "ipv4_address": "192.168.1.205",
                "ipv6_address": "fd06:fb62:d251:9033:0:0:0:33",
                "hostname": current_host
            }))
            .test();
    }

    /// Test for common configuration of ipv4 only (no ipv6)
    #[test]
    fn test_get_network_ipv4only() {
        let current_host = hostname::get().unwrap_or_else(|_| OsString::from("unknown"));

        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(
                PiholeFile::SetupVars,
                "IPV4_ADDRESS=192.168.1.205/24\n\
                 IPV6_ADDRESS=\n\
                 PIHOLE_INTERFACE=eth0\n",
            )
            .expect_json(json!({
                "interface": "eth0",
                "ipv4_address": "192.168.1.205",
                "ipv6_address": "",
                "hostname": current_host
            }))
            .test();
    }
}
