// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Program Main
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use std::process::exit;

fn main() {
    if let Err(e) = pihole_api::handle_cli() {
        e.print_stacktrace();
        exit(1);
    }
}
