// supported device list
// - srx1500

use crate::device::Device;
use regex::Regex;

pub const SUPPORTED_DEVICES: [&'static str; 2] = ["qfx5200", "srx1500"];

pub fn srx1500(name: &str) -> Device {
    let iface_patterns = vec![
        // ge-0/0/0 - ge-3/3/99
        r"^ge-[0-3]/[0-3]/\d{1,2}$",
        // xe-0/0/0 - xe-3/3/99
        r"^xe-[0-3]/[0-3]/\d{1,2}$",
        //  ae0 - ae999.999, lo0 - lo999.999
        r"^(ae|lo)\d{1,3}(\.\d{1,3})?$",
    ];

    Device::new(
        String::from(name),
        String::from("junos"),
        String::from("srx1500"),
        iface_patterns
            .iter()
            .map(|pattern| Regex::new(pattern).unwrap())
            .collect(),
    )
}

#[allow(dead_code)]
pub fn qfx5200(name: &str) -> Device {
    let iface_patterns = vec![
        // ge-0/0/0 - ge-3/3/99
        r"^ge-[0-3]/[0-3]/\d{1,2}$",
        // xe-0/0/0 - xe-3/3/99
        r"^xe-[0-3]/[0-3]/\d{1,2}$",
        //  ae0 - ae999.999, lo0 - lo999.999
        r"^(ae|lo)\d{1,3}(\.\d{1,3})?$",
    ];

    Device::new(
        String::from(name),
        String::from("junos"),
        String::from("qfx5200"),
        iface_patterns
            .iter()
            .map(|pattern| Regex::new(pattern).unwrap())
            .collect(),
    )
}
