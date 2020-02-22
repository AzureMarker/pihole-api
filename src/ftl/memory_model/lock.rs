// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Lock Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

/// The lock structure used to synchronize access to shared memory
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FtlLock {
    pub lock: libc::pthread_mutex_t,
    pub ftl_waiting_for_lock: bool
}
