use crate::device::{Device, SupportedPlatform};
use regex::Regex;
use std::fmt;
use std::str::FromStr;

/// ## Function
/// Returns a `device::Device`` after checking model is supported
/// - ingress & egress will be empty
pub fn new(name: &'static str, model: &str) -> Result<Device, String> {
    let new_model: SupportedModel = SupportedModel::from_str(model).unwrap();

    Ok(Device::new(
        name,
        SupportedPlatform::from_str("junos").unwrap(),
        new_model.to_string(),
        vec![],
        vec![],
        new_model.set_ifaces(),
    ))
}

/// ## Function
/// shorthand transform for `Vec<&str>` -> `Vec<Regex>`
fn as_regex(patterns: Vec<&str>) -> Vec<Regex> {
    patterns
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .collect()
}

/// ## Enum
/// This is a list of the full support device models for Junos.
/// ## Adding Devices
/// When adding devices be sure to update:
/// - implementations of `SupportedModel`
/// - create `set_<model>_ifaces` returning a `Vec<Regex>` with supported iface patterns
#[derive(Debug)]
pub enum SupportedModel {
    Qfx5200,
    Srx1500,
    Ptx1000,
}

impl SupportedModel {
    pub const ERROR_MSG: &'static str = "ModelNotSupported";
    pub const HELP_MSG: &'static str = "See `Feature Requests` for additional device support";

    fn set_ifaces(self) -> Vec<Regex> {
        match self {
            SupportedModel::Qfx5200 => set_qfx5200_ifaces(),
            SupportedModel::Srx1500 => set_srx1500_ifaces(),
            SupportedModel::Ptx1000 => set_ptx1000_ifaces(),
        }
    }
}

impl FromStr for SupportedModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "qfx5200" => Ok(SupportedModel::Qfx5200),
            "srx1500" => Ok(SupportedModel::Srx1500),
            "ptx1000" => Ok(SupportedModel::Ptx1000),
            _ => Err(format!(
                "{}: {}",
                SupportedModel::ERROR_MSG,
                SupportedModel::HELP_MSG
            )),
        }
    }
}

impl fmt::Display for SupportedModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = match self {
            SupportedModel::Qfx5200 => "qfx5200",
            SupportedModel::Srx1500 => "srx1500",
            SupportedModel::Ptx1000 => "ptx1000",
        };

        write!(f, "{}", s)
    }
}

fn set_srx1500_ifaces() -> Vec<Regex> {
    as_regex(vec![
        // ge-0/0/0 - ge-3/3/99
        r"^ge-[0-3]/[0-3]/\d{1,2}$",
        // xe-0/0/0 - xe-3/3/99
        r"^xe-[0-3]/[0-3]/\d{1,2}$",
        //  ae0 - ae999.999, lo0 - lo999.999
        r"^(ae|lo)\d{1,3}(\.\d{1,3})?$",
    ])
}

fn set_qfx5200_ifaces() -> Vec<Regex> {
    as_regex(vec![
        // ge-0/0/0 - ge-3/3/99
        r"^ge-[0-3]/[0-3]/\d{1,2}$",
        // xe-0/0/0 - xe-3/3/99
        r"^xe-[0-3]/[0-3]/\d{1,2}$",
        //  ae0 - ae999.999, lo0 - lo999.999
        r"^(ae|lo)\d{1,3}(\.\d{1,3})?$",
    ])
}

fn set_ptx1000_ifaces() -> Vec<Regex> {
    as_regex(vec![
        // ge-0/0/0 - ge-3/3/99
        r"^ge-[0-3]/[0-3]/\d{1,2}$",
        // xe-0/0/0 - xe-3/3/99
        r"^xe-[0-3]/[0-3]/\d{1,2}$",
        //  ae0 - ae999.999, lo0 - lo999.999
        r"^(ae|lo)\d{1,3}(\.\d{1,3})?$",
    ])
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    /// Required Type Checking is not until as Regex is compiled at
    fn assert_type<T>(_: &T) {}

    #[test]
    fn srx1500_valid_regex() {
        assert_type::<Vec<Regex>>(&super::set_srx1500_ifaces());
    }
    #[test]
    fn qfx5200_valid_regex() {
        assert_type::<Vec<Regex>>(&super::set_qfx5200_ifaces());
    }
    #[test]
    fn ptx1000_valid_regex() {
        assert_type::<Vec<Regex>>(&super::set_ptx1000_ifaces());
    }
}
