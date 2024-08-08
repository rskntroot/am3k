use crate::{crit, dbug, verb, LogLevel};
use regex::Regex;
use serde::Deserialize;
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub deployment: Deployment,
    pub defaults: Defaults,
}

impl Configuration {
    /// - loads an acl configuration from yaml
    /// - checks devices are valid
    /// - checks
    pub fn new(
        file_path: &str,
        acls_path: &str,
        dbg: LogLevel,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let mut valid_config: bool = true;
        let cfg: Configuration =
            serde_yml::from_str(&fs::read_to_string(PathBuf::from(file_path))?)?;
        dbug!(dbg, "{:#?}", cfg);

        verb!(dbg, "  Checking devicelist naming convention...");
        match are_names_complaint(&cfg.deployment.devicelist, &cfg.defaults.device_regex, dbg) {
            true => verb!(dbg, "  Devices matched convention."),
            false => valid_config = false,
        }

        verb!(dbg, "\n  Checking ruleset files exist...");
        match do_rulesets_exist(&cfg.deployment.rulesets, &acls_path, dbg) {
            true => verb!(dbg, "  Ruleset files exist."),
            false => valid_config = false,
        }

        match valid_config {
            true => Ok(Some(cfg)),
            false => Ok(None),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Deployment {
    pub rulesets: Vec<String>,
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
    #[serde(with = "regex_serde")]
    pub device_regex: Regex,
}

#[derive(Debug, Error)]
pub enum ConfigInvalid {
    #[error("DeviceNamesInvalid: failed to match provided regular expression")]
    DeviceNamesInvalid,
    #[error("RulesetFileDNE: failed to find matching ruleset file")]
    RulesetFileDNE,
    #[error(
        "FailedPostChecks: Loaded, but failed on DeviceNamesInvalid and/or RulesetFileDoesNotExist"
    )]
    FailedPostChecks,
}

/// regex lookup for devices against provided pattern
fn are_names_complaint(devicelist: &Vec<String>, pattern: &Regex, dbg: LogLevel) -> bool {
    let mut name_valid = true;
    for device in devicelist {
        if !pattern.is_match(&device) {
            crit!(dbg, "* {}: {}", ConfigInvalid::DeviceNamesInvalid, &device);
            name_valid = false;
        };
    }
    name_valid
}

/// pathbuf check on all rulesets
fn do_rulesets_exist(files: &Vec<String>, acls_path: &str, dbg: LogLevel) -> bool {
    let mut files_exist: bool = true;
    for file in files {
        if !PathBuf::from(format!("{acls_path}/{file}.acl")).exists() {
            crit!(dbg, "* {}: {}", ConfigInvalid::RulesetFileDNE, file);
            files_exist = false;
        }
    }
    files_exist
}

mod regex_serde {
    #![allow(dead_code)]
    use regex::Regex;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Regex::new(&s)
            .map_err(|e| de::Error::custom(format!("Invalid regular expression pattern: {}", e)))
    }

    pub fn serialize<S>(pattern: Regex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(pattern.as_str())
    }
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

        assert!(are_names_complaint(&devicelist, &pattern, dbg));
    }

    #[test]
    fn device_has_invalid_name() {
        let dbg: LogLevel = LogLevel::Debug;
        let devicelist: Vec<String> = vec![String::from("firewall1")];
        let pattern: Regex = Regex::new(
            "^[a-z]{1,3}([0-9]{1, 10}-){1,2}([a-z]{2, 9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0, 9})?",
        )
        .unwrap();

        assert_eq!(are_names_complaint(&devicelist, &pattern, dbg), false);
    }
}
