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
        load_ftl_db_config, load_gravity_db_config
    },
    env::{Config, Env},
    ftl::FtlMemory,
    routes::{
        auth::{self, AuthData},
        dns, settings, stats, version, web
    },
    services::PiholeModule,
    settings::{ConfigEntry, SetupVarsEntry},
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use rocket::{
    config::{ConfigBuilder, Environment},
    Rocket
};
use rocket_cors::CorsOptions;
use shaku::{Container, ContainerBuilder};

#[cfg(test)]
use rocket::config::LoggingLevel;

#[catch(404)]
fn not_found() -> Error {
    Error::from(ErrorKind::NotFound)
}

#[catch(401)]
fn unauthorized() -> Error {
    Error::from(ErrorKind::Unauthorized)
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() -> Result<(), Error> {
    let config = Config::load()?;
    let env = Env::Production(config);
    let key = SetupVarsEntry::WebPassword.read(&env)?;

    println!("{:#?}", env.config());

    let container = ContainerBuilder::new()
        .with_component_parameters::<GravityDatabasePool>(GravityDatabasePoolParameters {
            pool: CustomSqliteConnection::pool(load_gravity_db_config(&env)?)
                .context(ErrorKind::GravityDatabase)?
        })
        .with_component_parameters::<FtlDatabasePool>(FtlDatabasePoolParameters {
            pool: CustomSqliteConnection::pool(load_ftl_db_config(&env)?)
                .context(ErrorKind::FtlDatabase)?
        })
        .with_component_parameters::<Env>(env.clone())
        .build();

    setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Production)
                .address(env.config().general.address.as_str())
                .port(env.config().general.port as u16)
                .log_level(env.config().general.log_level)
                .finalize()
                .unwrap()
        ),
        FtlMemory::production(),
        env.config(),
        if key.is_empty() { None } else { Some(key) },
        container
    )
    .launch();

    Ok(())
}

/// Setup the API with the testing data and return a Client to test with
#[cfg(test)]
pub fn test(
    ftl_memory: FtlMemory,
    config: &Config,
    api_key: Option<String>,
    container: Container<'static, PiholeModule>
) -> Rocket {
    setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .log_level(LoggingLevel::Debug)
                .finalize()
                .unwrap()
        ),
        ftl_memory,
        &config,
        api_key,
        container
    )
}

/// General server setup
fn setup(
    server: Rocket,
    ftl_memory: FtlMemory,
    config: &Config,
    api_key: Option<String>,
    container: Container<'static, PiholeModule>
) -> Rocket {
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
            &web_route,
            routes![web::web_interface_index, web::web_interface]
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
        .register(catchers![not_found, unauthorized])
        // Manage the FTL shared memory configuration
        .manage(ftl_memory)
        // Manage the API key
        .manage(AuthData::new(api_key))
        // Manage the scheduler
        .manage(scheduler)
        // Manage the dependency injection container
        .manage(container)
        // Mount the API
        .mount(&api_mount_path_str, routes![
            version::version,
            auth::check,
            auth::logout,
            stats::get_summary,
            stats::top_domains,
            stats::top_clients,
            stats::upstreams,
            stats::query_types,
            stats::history,
            stats::recent_blocked,
            stats::clients,
            stats::over_time_history,
            stats::over_time_clients,
            stats::database::get_summary_db,
            stats::database::over_time_clients_db,
            stats::database::over_time_history_db,
            stats::database::query_types_db,
            stats::database::top_clients_db,
            stats::database::top_domains_db,
            stats::database::upstreams_db,
            dns::get_whitelist,
            dns::get_blacklist,
            dns::get_regexlist,
            dns::status,
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
