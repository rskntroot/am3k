#![allow(dead_code)]

use std::fmt;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
pub enum FieldError {
    ActionInvalid,
    ProtocolUnsupported,
    PortInvalid,
    PortOrderInvalid,
    RuleLengthErr,
    RuleExpansionUnsupported,
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg: (&str, &str) = match self {
            FieldError::ActionInvalid => (
                "ActionInvalid",
                "expected 'allow', 'deny', 'allowlog', or 'denylog'"),
            FieldError::ProtocolUnsupported => (
                "ProtocolUnsupported",
                "expected 'ip', 'tcp', 'udp', or 'icmp'"),
            FieldError::PortInvalid => (
                "PortInvalid",
                "expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'"),
            FieldError::PortOrderInvalid => (
                "PortOrderInvalid",
                "port range start must be less than port range end."
            ),
            FieldError::RuleLengthErr => (
                "RuleLengthErr",
                "expected 6 fields"),
            FieldError::RuleExpansionUnsupported => (
                "RuleExpansionUnsupported",
                "both src & dst ports cannot be port lists",
            ),
        };
        write!(f, "{}: {}", msg.0, msg.1)
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Action {
    Allow,
    Deny,
    AllowLog,
    DenyLog,
}

impl FromStr for Action {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allow" => Ok(Action::Allow),
            "deny" => Ok(Action::Deny),
            "allowlog" => Ok(Action::AllowLog),
            "denylog" => Ok(Action::DenyLog),
            _ => Err(FieldError::ActionInvalid),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            Action::Allow => "allow",
            Action::Deny => "deny",
            Action::AllowLog => "allowlog",
            Action::DenyLog => "denylog",
        };
        write!(f, "{}", description)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    IP,
}

impl FromStr for Protocol {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::TCP),
            "udp" => Ok(Protocol::UDP),
            "icmp" => Ok(Protocol::ICMP),
            "ip" => Ok(Protocol::IP),
            _ => Err(FieldError::ProtocolUnsupported),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::TCP => "tcp",
                Protocol::UDP => "udp",
                Protocol::ICMP => "icmp",
                Protocol::IP => "ip",
            }
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PortMap(Vec<(u16, u16)>);

impl PortMap {
    fn new() -> Self {
        PortMap(vec![])
    }

    /// ## Example PortMaps
    /// #### Single Value
    /// - "22" => `Port[(22,22)]`
    /// #### Range
    /// - "9000-9010" => `Port[9000,9010)]`
    /// #### List with Range
    /// - "80,443,9000-9010" => `Port[(80,80);(443,443);(9000,9010)]`
    fn from_str(s: &str) -> Result<Self, FieldError> {
        match s.parse::<u16>() {
            Ok(n) => Ok(PortMap::from_num(n)),
            Err(_) => {
                if s.contains(",") {
                    return PortMap::from_list(s);
                } else if s.contains('-') {
                    return Ok(PortMap(vec![Self::parse_range(s)?]));
                } else {
                    return Err(FieldError::PortInvalid);
                }
            }
        }
    }

    fn from_num(n: u16) -> Self {
        PortMap(vec![(n, n)])
    }

    fn from_range(start: u16, end: u16) -> Self {
        PortMap(vec![(start, end)])
    }

    fn from_list(s: &str) -> Result<Self, FieldError> {
        let mut port_map: Vec<(u16, u16)> = vec![];
        for part in s.split(',').collect::<Vec<&str>>() {
            if part.contains('-') {
                port_map.push(Self::parse_range(part)?);
            } else {
                let n = match part.parse::<u16>() {
                    Ok(num) => num,
                    Err(_) => return Err(FieldError::PortInvalid),
                };
                port_map.push((n, n));
            }
        }
        Ok(PortMap(port_map))
    }

    fn parse_range(s: &str) -> Result<(u16, u16), FieldError> {
        let parts: Vec<u16> = match s
            .split('-')
            .map(|s| s.parse::<u16>())
            .collect::<Result<Vec<u16>, _>>()
        {
            Ok(vec) => vec,
            Err(_) => return Err(FieldError::PortInvalid),
        };
        if parts.len() != 2 {
            return Err(FieldError::PortInvalid);
        } else if parts[0] > parts[1] {
            return Err(FieldError::PortOrderInvalid);
        }
        Ok((parts[0], parts[1]))
    }
}

impl fmt::Display for PortMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|&(start, end)| {
                    if start == end {
                        format!("{}", start)
                    } else {
                        format!("{}-{}", start, end)
                    }
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PortType {
    Any,
    Map(PortMap),
}

impl FromStr for PortType {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "any" {
            Ok(PortType::Any)
        } else {
            Ok(PortType::Map(PortMap::from_str(s)?))
        }
    }
}

impl fmt::Display for PortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PortType::Any => write!(f, "any"),
            PortType::Map(map) => write!(f, "{}", map),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    action: Action,
    protocol: Protocol,
    src_prefix: String,
    src_port: PortType,
    dst_prefix: String,
    dst_port: PortType,
}

impl FromStr for Rule {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(FieldError::RuleLengthErr);
            //return Err(format!("{}: {}, got {}", "RuleLengthErr", "", parts.len(),));
        }

        if parts[3].contains(',') && parts[5].contains(',') {
            return Err(FieldError::RuleExpansionUnsupported);
            // Err(format!( "{}: {}, got {}", "PortExpansionUnsupported", "", parts.len(), ));
        }

        Ok(Rule {
            action: Action::from_str(parts[0])?,
            protocol: Protocol::from_str(parts[1])?,
            src_prefix: String::from(parts[2]),
            src_port: PortType::from_str(parts[3])?,
            dst_prefix: String::from(parts[4]),
            dst_port: PortType::from_str(parts[5])?,
        })
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}",
            self.action,
            self.protocol,
            self.src_prefix,
            self.src_port,
            self.dst_prefix,
            self.dst_port
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    fn new(line: usize, column: usize) -> Self {
        Location { line, column }
    }
}

// todo add "phase" for global, src, dst
// ie we want to know that portinvalid was thrown during src or dst processing
// action and protocol can be considered "global" and ignored
#[derive(Debug, PartialEq)]
pub struct RuleErrors(Vec<(FieldError, Location)>);

impl RuleErrors {
    pub fn new() -> Self {
        RuleErrors(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, error: FieldError, loc: Location) {
        self.0.push((error, loc));
    }
}

impl IntoIterator for RuleErrors {
    type Item = (FieldError, Location);
    type IntoIter = IntoIter<(FieldError, Location)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a RuleErrors {
    type Item = &'a (FieldError, Location);
    type IntoIter = std::slice::Iter<'a, (FieldError, Location)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ruleset(Vec<Rule>);

impl Ruleset {
    pub fn new() -> Self {
        Ruleset(Vec::new())
    }

    pub fn push(&mut self, rule: Rule) {
        self.0.push(rule);
    }

    /// ## Function
    /// parses rules from raw strings to validated rules that may require expansion
    /// ## Logic
    /// - if all rules are valid Returns `Ok(Vec<crate::ruleset::Rule>)`
    /// - else returns list a rules with errors Returns: `Err(Vec<(String, String, usize)>)`
    ///     - as `(<error>, <rule>, <index>)`
    pub fn from_vec(raw_rules: &Vec<String>) -> Result<Self, RuleErrors> {
        let mut ruleset: Ruleset = Ruleset::new();
        let mut errors: RuleErrors = RuleErrors::new();

        for (i, rule) in raw_rules.iter().enumerate() {
            match Rule::from_str(rule) {
                Ok(r) => ruleset.push(r),
                Err(e) => errors.push(e, Location::new(i, 0)),
            };
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(ruleset)
    }
}

impl IntoIterator for Ruleset {
    type Item = Rule;
    type IntoIter = IntoIter<Rule>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::ruleset::*;

    #[test]
    fn portmap_num_valid() {
        dbg!(PortMap::from_str("0").unwrap());
        dbg!(PortMap::from_str("22").unwrap());
        dbg!(PortMap::from_str("65535").unwrap());
    }

    #[test]
    fn portmap_num_invalid() {
        dbg!(PortMap::from_str("22s").unwrap_err());
        dbg!(PortMap::from_str("1000000").unwrap_err());
    }

    #[test]
    fn portmap_list_valid() {
        dbg!(PortMap::from_str("80,443").unwrap());
    }

    #[test]
    fn portmap_list_invalid() {
        dbg!(PortMap::from_str("80,443s").unwrap_err());
    }

    #[test]
    fn portmap_range_valid() {
        let pm = PortMap::from_str("9000-9010").unwrap();
        println!("prints as '{}'", pm);
        dbg!(pm);
    }

    #[test]
    fn portmap_range_invalid() {
        dbg!(PortMap::from_str("9000-9010s").unwrap_err());
        dbg!(PortMap::from_str("9000-9010-10000").unwrap_err());
        dbg!(PortMap::from_str("65535-0").unwrap_err());
    }

    #[test]
    fn portmap_rangelist_valid() {
        let pm = PortMap::from_str("22,80,443,8000-8010,9000-9010").unwrap();
        println!("{}", pm);
        dbg!(pm);
        dbg!(PortMap::from_str("9000-9010,65535").unwrap());
    }

    #[test]
    fn portlist_expansion_valid() {
        let rs: Vec<String> = vec![
            "allow udp outside any inside 161,162".to_string(),
            "allow tcp inside any outside 22,80,443,9000-9010".to_string(),
        ];
        dbg!(Ruleset::from_vec(&rs).unwrap());
    }

    #[test]
    fn portlist_expansion_invalid() {
        let rs: Vec<String> = vec![
            "allow udp outside any inside 161,,162".to_string(),
            "allow tcp inside 22,*,443,9000-9010 outside any".to_string(),
        ];
        dbg!(Ruleset::from_vec(&rs).unwrap_err());
    }

    #[test]
    fn rule_contains_multiple_lists() {
        let rs: Vec<String> = vec!["allow tcp inside 20,21 outside 9000,9010".to_string()];
        dbg!(Ruleset::from_vec(&rs).unwrap_err());
    }

    #[test]
    fn rule_lengths_invalid() {
        let ss: &str = "short rule.";
        assert_eq!(Rule::from_str(ss).unwrap_err(), FieldError::RuleLengthErr);

        let ls: &str = "this is an extra long rule, ok.";
        assert_eq!(Rule::from_str(ls).unwrap_err(), FieldError::RuleLengthErr);
    }

    #[test]
    fn action_parse_err() {
        let s: &str = "[failhere] ip inside any outside any";
        assert_eq!(Rule::from_str(s).unwrap_err(), FieldError::ActionInvalid);
    }

    #[test]
    fn protocol_parse_err() {
        let s: &str = "deny [failhere] inside any outside any";
        assert_eq!(
            Rule::from_str(s).unwrap_err(),
            FieldError::ProtocolUnsupported
        );
    }

    #[test]
    fn src_port_invalid() {
        let s: &str = "deny ip inside [failhere] outside any";
        assert_eq!(Rule::from_str(s).unwrap_err(), FieldError::PortInvalid);
    }

    #[test]
    fn dst_port_invalid() {
        let s: &str = "deny ip inside any outside failhere";
        assert_eq!(Rule::from_str(s).unwrap_err(), FieldError::PortInvalid);
    }
}
