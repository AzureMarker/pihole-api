// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Server Setup Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::{
        custom_connection::CustomSqliteConnection,
        ftl::{FtlDatabasePool, FtlDatabasePoolParameters},
        gravity::{GravityDatabasePool, GravityDatabasePoolParameters},
        load_ftl_db_config, load_gravity_db_config,
    },
    env::{Config, Env},
    ftl::FtlMemory,
    routes::{
        auth::{self, AuthData},
        dns, settings, stats, version, web,
    },
    services::PiholeModule,
    settings::{ConfigEntry, SetupVarsEntry},
    util::{Error, ErrorKind},
};
use failure::ResultExt;
use rocket::{Build, Rocket};
use rocket_cors::CorsOptions;

#[cfg(test)]
use rocket::config::LogLevel;
use std::path::Path;

#[catch(404)]
fn not_found() -> Error {
    Error::from(ErrorKind::NotFound)
}

#[catch(401)]
fn unauthorized() -> Error {
    Error::from(ErrorKind::Unauthorized)
}

/// Run the API normally (connect to FTL over the socket)
pub async fn start(config_location: &Path) -> Result<(), Error> {
    let config = Config::load(config_location)?;
    let env = Env::Production(config);
    let key = SetupVarsEntry::WebPassword.read(&env)?;

    println!("{:#?}", env.config());

    let module = PiholeModule::builder()
        .with_component_parameters::<GravityDatabasePool>(GravityDatabasePoolParameters {
            pool: CustomSqliteConnection::pool(load_gravity_db_config(&env)?)
                .context(ErrorKind::GravityDatabase)?,
        })
        .with_component_parameters::<FtlDatabasePool>(FtlDatabasePoolParameters {
            pool: CustomSqliteConnection::pool(load_ftl_db_config(&env)?)
                .context(ErrorKind::FtlDatabase)?,
        })
        .with_component_parameters::<Env>(env.clone())
        .build();

    setup(
        rocket::custom(rocket::Config {
            address: env.config().general.address.parse().unwrap(),
            port: env.config().general.port as u16,
            log_level: env.config().general.log_level,
            ..Default::default()
        }),
        FtlMemory::production(),
        env.config(),
        if key.is_empty() { None } else { Some(key) },
        module,
    )
    .launch()
    .await
    .context(ErrorKind::Unknown)?;

    Ok(())
}

/// Setup the API with the testing data and return a Client to test with
#[cfg(test)]
pub fn test(
    ftl_memory: FtlMemory,
    config: &Config,
    api_key: Option<String>,
    module: PiholeModule,
) -> Rocket<Build> {
    setup(
        rocket::custom(rocket::Config {
            log_level: LogLevel::Debug,
            ..rocket::Config::debug_default()
        }),
        ftl_memory,
        &config,
        api_key,
        module,
    )
}

/// General server setup
fn setup(
    server: Rocket<Build>,
    ftl_memory: FtlMemory,
    config: &Config,
    api_key: Option<String>,
    module: PiholeModule,
) -> Rocket<Build> {
    // Set up CORS
    let cors = CorsOptions {
        allow_credentials: true,
        ..CorsOptions::default()
    }
    .to_cors()
    .unwrap();

    // Conditionally enable and mount the web interface
    let server = if config.web.enabled {
        let web_route = config.web.path.to_string_lossy();

        // Check if the root redirect should be enabled
        let server = if config.web.root_redirect && web_route != "/" {
            server.mount("/", routes![web::web_interface_redirect])
        } else {
            server
        };

        // Mount the web interface at the configured route
        server.mount(
            web_route.as_ref(),
            routes![web::web_interface_index, web::web_interface],
        )
    } else {
        server
    };

    // The path to mount the API on (always <web_root>/api)
    let mut api_mount_path = config.web.path.clone();
    api_mount_path.push("api");
    let api_mount_path_str = api_mount_path.to_string_lossy();

    // Create a scheduler for scheduling work (ex. disable for 10 minutes)
    let scheduler = task_scheduler::Scheduler::new();

    // Set up the server
    server
        // Attach CORS handler
        .attach(cors)
        // Add custom error handlers
        .register("/", catchers![not_found, unauthorized])
        // Manage the FTL shared memory configuration
        .manage(ftl_memory)
        // Manage the API key
        .manage(AuthData::new(api_key))
        // Manage the scheduler
        .manage(scheduler)
        // Manage the dependency injection module
        .manage(Box::new(module))
        // Mount the API
        .mount(api_mount_path_str.as_ref(), routes![
            version::version,
            auth::check,
            auth::logout,
            stats::summary::get_summary,
            stats::top_domains::route,
            stats::top_clients::route,
            stats::upstreams::route,
            stats::query_types::route,
            stats::history::route,
            stats::recent_blocked::route,
            stats::clients::route,
            stats::over_time_history::route,
            stats::over_time_clients::route,
            stats::database::summary_db::get_summary_db,
            stats::database::over_time_clients_db::route,
            stats::database::over_time_history_db::route,
            stats::database::query_types_db::route,
            stats::database::top_clients_db::route,
            stats::database::top_domains_db::route,
            stats::database::upstreams_db::route,
            dns::get_whitelist,
            dns::get_blacklist,
            dns::get_regexlist,
            dns::get_status,
            dns::change_status,
            dns::add_whitelist,
            dns::add_blacklist,
            dns::add_regexlist,
            dns::delete_whitelist,
            dns::delete_blacklist,
            dns::delete_regexlist,
            settings::get_dhcp,
            settings::put_dhcp,
            settings::get_dns,
            settings::put_dns,
            settings::get_ftldb,
            settings::get_ftl,
            settings::get_network,
            settings::get_web,
            settings::put_web
        ])
}
