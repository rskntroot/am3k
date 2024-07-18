use regex::Regex;

// ============== JUNOS SUPPORTED DEVICES ==============

#[derive(Debug)]
pub struct Srx1500 {
    pub device_name: &'static str,
    pub valid_ifaces: Vec<Regex>,
}

impl Srx1500 {
    pub fn new() -> Self {
        let iface_regex = [
            // Loopback interfaces, e.g., lo0, lo1
            r"^lo\d+$",
            // Gigabit Ethernet interfaces, e.g., ge-0/0/0
            r"^ge-[0-3]/[0-3]/\d$",
            // 10 Gigabit Ethernet interfaces, e.g., xe-0/0/0
            r"^xe-[0-3]/[0-3]/\d+$",
            // Aggregated Ethernet interfaces, e.g., ae0 - ae999.999
            r"^ae\d{1,3}(\.\d{1,3})?$",
        ];

        Srx1500 {
            device_name: "srx1500",
            valid_ifaces: compile_regexes(iface_regex),
        }
    }
}

pub fn is_valid_iface(iface: &str, valid_ifaces: Vec<Regex>) -> Result<(bool, String), String> {
    for exp in valid_ifaces {
        if exp.is_match(iface) {
            return Ok((true, exp.to_string()));
        }
    }

    Ok((false, String::new()))
}

fn compile_regexes(raw_regexes: [&str; 4]) -> Vec<Regex> {
    let mut compiled_regexes = Vec::with_capacity(raw_regexes.len());

    for regex_str in raw_regexes.iter() {
        match Regex::new(regex_str) {
            Ok(regex) => {
                compiled_regexes.push(regex);
            }
            Err(err) => {
                panic!("Error compiling regex `{}`: {}", regex_str, err);
            }
        }
    }

    compiled_regexes
}
