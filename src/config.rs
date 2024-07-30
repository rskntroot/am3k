use crate::{dbug, info, LogLevel};

use regex::Regex;
use serde::Deserialize;
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub ruleset: Vec<String>,
    pub deployment: Deployment,
    pub defaults: Defaults,
}

impl Configuration {
    /// Loads an acl configuration from yaml and checks devices are valid
    pub fn new(file_path: &str, dbg: LogLevel) -> Result<Self, Box<dyn std::error::Error>> {
        info!(dbg, "\nLoading configuration file {}...", file_path);
        let cfg: Configuration =
            serde_yml::from_str(&fs::read_to_string(PathBuf::from(file_path))?)?;
        info!(dbg, "Configuration file loaded successfully from yaml.");
        dbug!(dbg, "{:#?}", cfg);

        info!(dbg, "\nChecking devicelist naming convention...");
        match check_device_names(&cfg.deployment.devicelist, &cfg.defaults.device_regex) {
            Ok(_) => info!(dbg, "Valid device names per naming convention."),
            Err(_) => return Err(Box::new(ConfigInvalid::DeviceNameInvalid)),
        };

        Ok(cfg)
    }
}

#[derive(Debug, Deserialize)]
pub struct Deployment {
    pub platform: String,
    pub model: String,
    pub devicelist: Vec<String>,
    pub ingress: Direction,
    pub egress: Direction,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Direction {
    pub interfaces: Vec<String>,
    pub filters: Filters,
    pub deployable: bool,
    pub established: bool,
    pub default: String,
    pub transforms: Transforms,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Filters {
    pub src: Vec<String>,
    pub dst: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Transforms {
    pub src: bool,
    pub dst: bool,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub device_regex: String,
}

#[derive(Debug, Error)]
pub enum ConfigInvalid {
    #[error("Device failed to match provided regular expression.")]
    DeviceNameInvalid,
}

/// Regex lookup for all devices in provided `devicelist` against provided `pattern`
/// - if all devices in devicelist match the provided pattern eturns `Result: Ok(true)`
/// - else returns list of devices with DeviceNameInvalid `Result: Err(Vec<String>)`
/// - note: does not return `false`
fn check_device_names(devicelist: &Vec<String>, pattern: &str) -> Result<bool, Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let pattern: Regex = match Regex::new(pattern) {
        Ok(exp) => exp,
        Err(_) => {
            errors.push(String::from("InvalidRegexExpression"));
            return Err(errors);
        }
    };

    for device in devicelist {
        match pattern.is_match(&device) {
            true => {}
            _ => errors.push(format!("DeviceNameInvalid on device {}", &device)),
        };
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn device_has_valid_name() {
        let devicelist: Vec<String> = vec![String::from("rsk101-example-fw1")];
        let pattern: &str =
            "^[a-z]{1,3}([0-9]{1,10}-){1,2}([a-z]{2,9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0,9})?";

        let result: bool = super::check_device_names(&devicelist, pattern).unwrap();

        assert_eq!(result, true);
    }

    #[test]
    fn device_has_invalid_name() {
        let devicelist: Vec<String> = vec![String::from("firewall1")];
        let pattern: &str =
            "^[a-z]{1,3}([0-9]{1,10}-){1,2}([a-z]{2,9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0,9})?";

        let result: Vec<String> = super::check_device_names(&devicelist, pattern).unwrap_err();

        assert!(
            result[0].contains("DeviceNameInvalid"),
            "Device unexpectedly matched regex."
        );
    }
}
