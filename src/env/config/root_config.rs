// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Root Level Config
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::config::{file_locations::Files, general::General, web::WebConfig},
    util::{Error, ErrorKind}
};
use failure::{Fail, ResultExt};
use std::{
    fs::File,
    io::{self, prelude::*},
    path::Path
};

/// The default config location
pub const DEFAULT_CONFIG_LOCATION: &str = "/etc/pihole/API.toml";

/// The API config options
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub file_locations: Files,
    #[serde(default)]
    pub web: WebConfig
}

impl Config {
    /// Load the config from the file located at `config_location`. If it does
    /// not exist, return the default config.
    pub fn load(config_location: &Path) -> Result<Config, Error> {
        let mut buffer = String::new();

        // Read the file to a string, but return the default config if the file doesn't
        // exist
        let mut file = match File::open(config_location) {
            Ok(f) => f,
            Err(e) => {
                return match e.kind() {
                    io::ErrorKind::NotFound => {
                        println!(
                            "Cannot find config file {}, using default config",
                            config_location.display()
                        );
                        Ok(Self::default())
                    }
                    _ => Err(Error::from(e.context(ErrorKind::FileRead(
                        config_location.display().to_string()
                    ))))
                }
            }
        };

        file.read_to_string(&mut buffer).map_err(|e| {
            Error::from(e.context(ErrorKind::FileRead(config_location.display().to_string())))
        })?;

        let config = toml::from_str::<Config>(&buffer).context(ErrorKind::ConfigParsingError)?;

        if config.is_valid() {
            Ok(config)
        } else {
            Err(Error::from(ErrorKind::ConfigParsingError))
        }
    }

    /// Check if the config settings are valid
    pub fn is_valid(&self) -> bool {
        self.general.is_valid() && self.file_locations.is_valid() && self.web.is_valid()
    }
}

#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    fn valid_config() {
        let config = Config::default();
        assert!(config.is_valid());
    }
}
