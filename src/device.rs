use regex::Regex;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformUnsupported {
    #[error("See `Platform Onboarding` for additional platform support")]
    MakeNotSupported,
    #[error("See `Device Onboarding` for additional platform support")]
    ModelNotSupported,
}

#[derive(Debug)]
pub enum SupportedPlatform {
    Juniper,
}

impl std::str::FromStr for SupportedPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "junos" => Ok(SupportedPlatform::Juniper),
            _ => Err(format!("{}", PlatformUnsupported::MakeNotSupported)),
        }
    }
}

impl fmt::Display for SupportedPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SupportedPlatform::Juniper => "junos",
            }
        )
    }
}

#[derive(Debug)]
pub struct Device {
    pub name: &'static str,
    pub platform: SupportedPlatform,
    pub model: String,
    pub ingress: Vec<String>,
    pub egress: Vec<String>,
    iface_regex: Vec<Regex>,
}

impl Device {
    pub fn new(
        name: &'static str,
        platform: SupportedPlatform,
        model: String,
        ingress: Vec<String>,
        egress: Vec<String>,
        iface_regex: Vec<Regex>,
    ) -> Device {
        Device {
            name,
            platform,
            model,
            ingress,
            egress,
            iface_regex,
        }
    }

    /// Checks if a single provided `iface` is valid
    /// - if valid Returns `(bool: true, String: pattern)`
    /// - else Returns `false, String: "")`
    pub fn is_valid_iface(&self, iface: &str) -> Result<(bool, String), String> {
        for exp in &self.iface_regex {
            if exp.is_match(iface) {
                return Ok((true, exp.to_string()));
            }
        }

        Ok((false, String::new()))
    }

    /// Checks all interfaces using `crate::device::Device::is_valid_iface`
    /// - if all interfaces are valid, return `Ok(Vec<String>)` containing valid matches
    /// - else returns `Err(Vec<String>)` containing `InvalidPortAssigned` matches
    pub fn are_valid_ifaces(&self, interfaces: &Vec<String>) -> Result<Vec<String>, Vec<String>> {
        let mut valid_ifaces: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for iface in interfaces {
            match self.is_valid_iface(&iface) {
                Ok((true, str)) => {
                    valid_ifaces.push(format!(" - \'{}\' matched \'{}\'", iface, str))
                }
                Ok((false, _str)) => {
                    errors.push(format!("InvalidPortAssigned on interface \'{}\'", iface))
                }
                Err(e) => panic!("{}", e),
            }
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(valid_ifaces)
    }

    fn format_regex(&self) -> String {
        self.iface_regex
            .iter()
            .enumerate()
            .map(|(i, pattern)| format!("  {}: {}\n", i + 1, pattern))
            .collect()
    }

    fn format_ifaces(&self, direction: Direction) -> String {
        let selector = match direction {
            Direction::Ingress => &self.ingress,
            Direction::Egress => &self.egress,
        };

        selector
            .iter()
            .enumerate()
            .map(|(i, iface)| format!("  {}: {}\n", i + 1, iface))
            .collect::<String>()
    }
}

#[derive(Debug)]
enum Direction {
    Ingress,
    Egress,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "device Name: {}\nplatform: {}\nmodel: {}\ningress:\n{}\negress:\n{}\niface_regex:\n{}
            ",
            self.name,
            self.platform,
            self.model,
            self.format_ifaces(Direction::Ingress),
            self.format_ifaces(Direction::Egress),
            self.format_regex()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{Device, SupportedPlatform};
    use regex::Regex;

    #[test]
    fn device_has_valid_iface() {
        let iface: &str = "et-0/0/0";
        let pattern: &str = "et.*";
        let pattern: Regex = Regex::new(pattern).unwrap();

        let test_device: Device = Device::new(
            "test-device",
            SupportedPlatform::from_str("junos").unwrap(),
            String::from("srx1500"),
            vec![],
            vec![],
            vec![pattern],
        );

        let result: bool = test_device.is_valid_iface(iface).unwrap().0;
        assert_eq!(result, true)
    }

    #[test]
    fn device_has_invalid_iface() {
        let iface: &str = "et-0/0/0";
        let pattern: &str = "^xe";
        let pattern: Regex = Regex::new(pattern).unwrap();

        let test_device: Device = Device::new(
            "test-device",
            SupportedPlatform::from_str("junos").unwrap(),
            String::from("srx1500"),
            vec![],
            vec![],
            vec![pattern],
        );

        let result: bool = test_device.is_valid_iface(iface).unwrap().0;
        assert_eq!(result, false)
    }
}
