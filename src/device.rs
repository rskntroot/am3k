use regex::Regex;

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
