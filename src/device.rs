use regex::Regex;
use std::fmt;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub make: String,
    pub model: String,
    pub interfaces: Vec<Regex>,
}

impl Device {
    pub fn new(name: String, make: String, model: String, interfaces: Vec<Regex>) -> Device {
        Device {
            name,
            make,
            model,
            interfaces,
        }
    }

    pub fn is_valid_iface(&self, iface: &str) -> Result<(bool, String), String> {
        for exp in &self.interfaces {
            if exp.is_match(iface) {
                return Ok((true, exp.to_string()));
            }
        }

        Ok((false, String::new()))
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Device Name: {}\nMake: {}\nModel: {}\nInterfaces:\n{}",
            self.name,
            self.make,
            self.model,
            self.interfaces
                .iter()
                .enumerate()
                .map(|(i, iface)| format!("  {}: {}\n", i + 1, iface))
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Device;
    use regex::Regex;

    #[test]
    fn device_has_valid_iface() {
        let iface: &str = "et-0/0/0";
        let pattern: &str = "et.*";
        let pattern: Regex = Regex::new(pattern).expect("Failed to create regex");

        let test_device: Device = Device::new(
            String::from("test-device"),
            String::from("junos"),
            String::from("srx1500"),
            vec![pattern],
        );

        let result: bool = test_device.is_valid_iface(iface).unwrap().0;
        assert_eq!(result, true)
    }

    #[test]
    fn device_has_invalid_iface() {
        let iface: &str = "et-0/0/0";
        let pattern: &str = "^xe";
        let pattern: Regex = Regex::new(pattern).expect("Failed to create regex");

        let test_device: Device = Device::new(
            String::from("test-device"),
            String::from("junos"),
            String::from("srx1500"),
            vec![pattern],
        );

        let result: bool = test_device.is_valid_iface(iface).unwrap().0;
        assert_eq!(result, false)
    }

    #[test]
    fn device_has_valid_name() {
        let device_name: &str = "rsk101-example-fw1";
        let pattern: Regex = Regex::new(
            "^[a-z]{1,3}([0-9]{1,10}-){1,2}([a-z]{2,9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0,9})?",
        )
        .expect("Failed to create regex");

        let result: bool = pattern.is_match(device_name);
        assert!(
            result,
            "DeviceNameInvalid device {} with {}",
            device_name, pattern
        );
    }
}
