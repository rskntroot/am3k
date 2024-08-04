use crate::{crit, dbug, info, verb, LogLevel};
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
    /// loads an acl configuration from yaml and checks devices are valid
    pub fn new(file_path: &str, dbg: LogLevel) -> Result<Self, Box<dyn std::error::Error>> {
        info!(dbg, "\nLoading configuration file {}...", file_path);
        let cfg: Configuration =
            serde_yml::from_str(&fs::read_to_string(PathBuf::from(file_path))?)?;

        verb!(dbg, "  Checking devicelist naming convention...");
        let pattern: Regex = Regex::new(&cfg.defaults.device_regex.to_string())?;
        if device_names_are_complaint(&cfg.deployment.devicelist, pattern, dbg)? {
            verb!(dbg, "  Valid device names per naming convention.")
        };
        dbug!(dbg, "{:#?}", cfg);
        info!(dbg, "Configuration file loaded successfully from yaml.");

        Ok(cfg)
    }
}

#[derive(Debug, Deserialize)]
pub struct Deployment {
    pub platform: Platform,
    pub devicelist: Vec<String>,
    pub ingress: Direction,
    pub egress: Direction,
}

#[derive(Debug, Deserialize)]
pub struct Platform {
    pub make: String,
    pub model: String,
}

#[allow(dead_code)] // todo
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
    #[error("DeviceNamesInvalid: failed to match provided regular expression.")]
    DeviceNamesInvalid,
}

/// regex lookup for devices against provided pattern
fn device_names_are_complaint(
    devicelist: &Vec<String>,
    pattern: Regex,
    dbg: LogLevel,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut invalid_name_detected = false;
    for device in devicelist {
        if !pattern.is_match(&device) {
            crit!(
                dbg,
                "{} on device {}",
                ConfigInvalid::DeviceNamesInvalid,
                &device
            );
            invalid_name_detected = true;
        };
    }

    if invalid_name_detected {
        return Err(Box::new(ConfigInvalid::DeviceNamesInvalid));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_has_valid_name() {
        let dbg = LogLevel::Debug;
        let devicelist: Vec<String> = vec![String::from("rsk101-example-fw1")];
        let pattern: Regex = Regex::new(
            "^[a-z]{1,3}([0-9]{1, 10}-){1,2}([a-z]{2, 9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0, 9})?",
        )
        .unwrap();

        assert!(device_names_are_complaint(&devicelist, pattern, dbg).unwrap());
    }

    #[test]
    fn device_has_invalid_name() {
        let dbg: LogLevel = LogLevel::Debug;
        let devicelist: Vec<String> = vec![String::from("firewall1")];
        let pattern: Regex = Regex::new(
            "^[a-z]{1,3}([0-9]{1, 10}-){1,2}([a-z]{2, 9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0, 9})?",
        )
        .unwrap();

        assert_eq!(
            device_names_are_complaint(&devicelist, pattern, dbg)
                .unwrap_err()
                .to_string(),
            ConfigInvalid::DeviceNamesInvalid.to_string()
        );
    }
}
