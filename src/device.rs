use crate::{crit, dbug, verb, LogLevel};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct SupportedPlatform {
    pub make: String,
    pub models: Vec<Models>,
}

impl SupportedPlatform {
    pub fn from_file(file_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_yml::from_str(&fs::read_to_string(PathBuf::from(
            file_path,
        ))?)?)
    }

    pub fn lookup_model_regex(&self, model_name: &str) -> Option<&Vec<Regex>> {
        self.models
            .iter()
            .find(|model| model.name == model_name)
            .map(|model| &model.interfaces)
    }
}

impl fmt::Display for SupportedPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:\n{:#?}", self.make, self.models)
    }
}

#[derive(Debug, Deserialize)]
pub struct Models {
    pub name: String,
    #[serde(with = "regex_serde")]
    pub interfaces: Vec<Regex>,
}

impl fmt::Display for Models {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {}\nifaces: {:?}", self.name, self.interfaces)
    }
}

#[derive(Debug, Error)]
pub enum PlatformUnsupported {
    #[error("MakeNotSupported: see `Platform Onboarding` for more information")]
    MakeNotSupported,
    #[error("ModelNotSupported: see `Device Onboarding` for more information")]
    ModelNotSupported,
}

#[derive(Debug, Serialize)]
pub struct Device {
    pub name: String,
    pub make: String,
    pub model: String,
    pub paths: Paths,
}

impl Device {
    pub fn build(
        name: &str,
        make: &str,
        model: &str,
        ingress: &Vec<String>,
        egress: &Vec<String>,
        platforms_path: &str,
        dbg: LogLevel,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        verb!(dbg, "  Loading path to supported platforms...");
        let dir = PathBuf::from(platforms_path);
        verb!(dbg, "  Found path: {}", &dir.display());

        verb!(dbg, "\n  Searching for matching supported platform file...");
        let file = match get_supported_platform_file(&dir, &make) {
            Ok(file) => file,
            Err(e) => {
                crit!(
                    dbg,
                    "  Unable to find supported platform [{}] in [{}]",
                    &make,
                    &dir.display()
                );
                return Err(e);
            }
        };
        verb!(dbg, "  Found {}", &file.display());

        verb!(dbg, "\n  Loading supported platforms file...");
        let platform_cfg: SupportedPlatform = SupportedPlatform::from_file(&file)?;
        verb!(dbg, "  Platforms file loaded successfully from yaml.");

        verb!(dbg, "\n  Checking supported model...");
        let patterns = match platform_cfg.lookup_model_regex(model) {
            Some(regex) => regex,
            None => {
                crit!(
                    dbg,
                    "  Unable to find supported model [{}] in [{}]",
                    &model,
                    &dir.display()
                );
                return Err(Box::new(PlatformUnsupported::ModelNotSupported));
            }
        };
        verb!(dbg, "  Model supported.");

        Ok(Device {
            name: name.to_owned(),
            make: make.to_owned(),
            model: model.to_owned(),
            paths: Paths::build(ingress, egress, patterns, dbg)?,
        })
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "device Name: {}\nplatform: {}\nmodel: {}\npaths: {}",
            self.name, self.make, self.model, self.paths,
        )
    }
}

#[derive(Debug, Serialize)]
pub struct Paths {
    pub ingress: Vec<String>,
    pub egress: Vec<String>,
}

impl Paths {
    fn build(
        ingress: &Vec<String>,
        egress: &Vec<String>,
        patterns: &Vec<Regex>,
        dbg: LogLevel,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        verb!(dbg, "\n  Confirming interfaces are valid...");
        dbug!(dbg, "{:#?}", patterns);
        let mut invalid_ifaces_detected: bool = false;
        if let Some(ifaces) = Self::list_invalid_ifaces(ingress, &patterns) {
            crit!(
                dbg,
                "  Ingress{}: {:?}",
                InterfaceErrors::InvalidPortAssignment,
                ifaces
            );
            invalid_ifaces_detected = true;
        }
        if let Some(ifaces) = Self::list_invalid_ifaces(egress, &patterns) {
            crit!(
                dbg,
                "  Egress{}: {:?}",
                InterfaceErrors::InvalidPortAssignment,
                ifaces
            );
            invalid_ifaces_detected = true;
        }
        if invalid_ifaces_detected {
            return Err(Box::new(InterfaceErrors::InvalidPortAssignment));
        }
        verb!(dbg, "  Interfaces are valid");

        Ok(Paths {
            ingress: ingress.to_owned(),
            egress: egress.to_owned(),
        })
    }

    /// return list of interfaces that dont match provided regexes
    fn list_invalid_ifaces(interfaces: &Vec<String>, patterns: &Vec<Regex>) -> Option<Vec<String>> {
        let errors: Vec<String> = interfaces
            .iter()
            .filter_map(|iface| {
                if !Self::is_valid_iface(iface, patterns) {
                    Some(iface.to_string())
                } else {
                    None
                }
            })
            .collect();

        match errors.len() > 0 {
            true => Some(errors),
            false => None,
        }
    }

    /// shorthand check single interface against provided regexes
    fn is_valid_iface(iface: &str, patterns: &Vec<Regex>) -> bool {
        for exp in patterns {
            if exp.is_match(iface) {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for Paths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ingress: {:?}\negress: {:?}", self.ingress, self.egress)
    }
}

fn get_supported_platform_file(
    path: &PathBuf,
    make: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match contains_yaml_files(path)? {
        Some(file_list) => {
            for file in file_list {
                if file.contains(make) {
                    return Ok(PathBuf::from(file));
                }
            }
        }
        None => (),
    }
    Err(Box::new(PlatformUnsupported::MakeNotSupported))
}

fn contains_yaml_files(path: &PathBuf) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(Some(
        entries
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.path().to_str().map(|s| s.to_owned()))
            })
            .collect(),
    ))
}

#[derive(Debug, Error)]
pub enum InterfaceErrors {
    #[error("InvalidPortAssignment: interfaces do not exist on provided platform")]
    InvalidPortAssignment,
}

mod regex_serde {
    #![allow(dead_code)]
    use regex::Regex;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Deserialize::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| Regex::new(&s).map_err(serde::de::Error::custom))
            .collect()
    }

    pub fn serialize<S>(regexes: &[Regex], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let strings: Vec<String> = regexes.iter().map(|r| r.as_str().to_owned()).collect();
        strings.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn build_device_succeeds() {
        let ports = vec!["xe-0/0/0".to_string(), "xe-0/0/1".to_string()];
        let dbg = crate::LogLevel::Debug;
        let _ = Device::build(
            "test-device",
            "juniper",
            "srx1500",
            &ports,
            &ports,
            "./platforms",
            dbg,
        );
    }

    #[test]
    fn build_path_errs_on_invalid_iface() {
        let ports = vec!["et-0/0/0".to_string(), "et-0/0/1".to_string()];
        let patterns = vec![Regex::new("^xe").unwrap()];
        let dbg = crate::LogLevel::Debug;
        assert_eq!(
            Paths::build(&ports, &ports, &patterns, dbg)
                .unwrap_err()
                .to_string(),
            InterfaceErrors::InvalidPortAssignment.to_string()
        );
    }
}
