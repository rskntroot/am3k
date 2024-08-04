#![allow(dead_code)]

use serde::Serialize;
use std::fmt;
use std::str::FromStr;
use std::vec::IntoIter;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum FieldError {
    #[error("ActionInvalid: expected 'allow', 'deny', 'allowlog', or 'denylog'")]
    ActionInvalid,
    #[error("ProtocolUnsupported: expected 'ip', 'tcp', 'udp', or 'icmp'")]
    ProtocolUnsupported,
    #[error("PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'")]
    PortInvalid,
    #[error("PortOrderInvalid: port range start must be less than port range end")]
    PortOrderInvalid,
    #[error("RuleLengthErr: expected 6 fields")]
    RuleLengthErr,
    #[error("RuleExpansionUnsupported: both src & dst ports cannot be port lists")]
    RuleExpansionUnsupported,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PortMap(Vec<(u16, u16)>);

impl PortMap {
    fn new() -> Self {
        PortMap(vec![])
    }

    /// parses a variable port string into range tuples
    /// - supports u16, range(u16), and list of u16/range(u16)
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

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_expandable(&self) -> bool {
        if self.len() > 1 {
            return true;
        }
        for t in self {
            if t.0 != t.1 {
                return true;
            }
        }
        return false;
    }
}

impl IntoIterator for PortMap {
    type Item = (u16, u16);
    type IntoIter = IntoIter<(u16, u16)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a PortMap {
    type Item = &'a (u16, u16);
    type IntoIter = std::slice::Iter<'a, (u16, u16)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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
                        format!("({})", start)
                    } else {
                        format!("({},{})", start, end)
                    }
                })
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PortType {
    Any,
    Map(PortMap),
    Port(u16),
}

impl PortType {
    fn new() -> Self {
        PortType::Map(PortMap(vec![(0, 0)]))
    }

    fn is_expandable(&self) -> bool {
        if let &PortType::Map(ref map) = &self {
            return map.is_expandable();
        }
        false
    }

    fn get_expansion(&self) -> Option<Vec<u16>> {
        if let PortType::Map(map) = self {
            let mut expanded_map: Vec<u16> = vec![];
            for port_map in map {
                expanded_map.push(port_map.0);
                if port_map.0 != port_map.1 {
                    expanded_map.push(port_map.1);
                }
            }
            return Some(expanded_map);
        }
        None
    }
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

impl Serialize for PortType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PortType::Any => serializer.serialize_str("any"),
            PortType::Map(map) => map.serialize(serializer),
            PortType::Port(num) => serializer.serialize_u16(*num),
        }
    }
}

impl fmt::Display for PortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PortType::Any => write!(f, "any"),
            PortType::Map(map) => write!(f, "{}", map),
            PortType::Port(num) => write!(f, "{}", num),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Rule {
    action: Action,
    protocol: Protocol,
    src_prefix: String,
    src_port: PortType,
    dst_prefix: String,
    dst_port: PortType,
}

impl Rule {
    pub fn expand(&self) -> Vec<Rule> {
        let mut expanded_rules: Vec<Rule> = vec![];

        if let Some(port_expansion) = self.src_port.get_expansion() {
            let mut rule_clone: Rule = self.clone();
            for port in port_expansion {
                rule_clone.src_port = PortType::Port(port);
                expanded_rules.push(rule_clone.clone());
            }
        } else if let Some(port_expansion) = self.dst_port.get_expansion() {
            let mut rule_clone: Rule = self.clone();
            for port in port_expansion {
                rule_clone.dst_port = PortType::Port(port);
                expanded_rules.push(rule_clone.clone());
            }
        } else {
            expanded_rules.push(self.clone());
        }

        expanded_rules
    }
}

impl FromStr for Rule {
    type Err = (FieldError, Location);

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 6 {
            return Err((FieldError::RuleLengthErr, Location::new(0, s.len() + 1)));
        }

        if parts[3].contains(',') && parts[5].contains(',') {
            return Err((
                FieldError::RuleExpansionUnsupported,
                Location::new(0, s.len() + 1),
            ));
        }

        let mut columns: Vec<usize> = vec![];
        for (i, c) in s.trim().char_indices() {
            if c.is_whitespace() {
                columns.push(i + 1);
            }
        }

        let action: Action = match Action::from_str(parts[0]) {
            Ok(action) => action,
            Err(e) => return Err((e, Location::new(0, 0))),
        };

        let protocol: Protocol = match Protocol::from_str(parts[1]) {
            Ok(protocol) => protocol,
            Err(e) => return Err((e, Location::new(0, columns[0]))),
        };

        // placeholder for src_prefix

        let src_port: PortType = match PortType::from_str(parts[3]) {
            Ok(protocol) => protocol,
            Err(e) => return Err((e, Location::new(0, columns[2]))),
        };

        // placeholder for dst_prefix

        let dst_port: PortType = match PortType::from_str(parts[5]) {
            Ok(protocol) => protocol,
            Err(e) => return Err((e, Location::new(0, columns[4]))),
        };

        Ok(Rule {
            action,
            protocol,
            src_prefix: String::from(parts[2]),
            src_port,
            dst_prefix: String::from(parts[4]),
            dst_port,
        })
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
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

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Ruleset(Vec<Rule>);

impl Ruleset {
    pub fn new() -> Self {
        Ruleset(Vec::new())
    }

    pub fn push(&mut self, rule: Rule) {
        self.0.push(rule);
    }

    /// parses rules from vec of strings to validated rules that may require expansion
    pub fn from_vec(raw_rules: &Vec<String>) -> Result<Self, RuleErrors> {
        let mut ruleset: Ruleset = Ruleset::new();
        let mut errors: RuleErrors = RuleErrors::new();

        for (i, rule) in raw_rules.iter().enumerate() {
            match Rule::from_str(rule) {
                Ok(r) => ruleset.push(r),
                Err((e, l)) => {
                    errors.push(e, Location::new(i, l.column));
                }
            };
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(ruleset)
    }

    pub fn expand(self) -> Self {
        let mut expanded_ruleset: Vec<Rule> = vec![];

        for rule in self {
            expanded_ruleset.append(&mut rule.expand());
        }

        Ruleset(expanded_ruleset)
    }
}

impl IntoIterator for Ruleset {
    type Item = Rule;
    type IntoIter = IntoIter<Rule>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for Ruleset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Ruleset(")?;
        for rule in &self.0 {
            writeln!(f, "  {}", rule)?;
        }
        write!(f, ")")
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
        assert_eq!(Rule::from_str(ss).unwrap_err().0, FieldError::RuleLengthErr);

        let ls: &str = "this is an extra long rule, ok.";
        assert_eq!(Rule::from_str(ls).unwrap_err().0, FieldError::RuleLengthErr);
    }

    #[test]
    fn action_parse_err() {
        let s: &str = "[failhere] ip inside any outside any";
        assert_eq!(Rule::from_str(s).unwrap_err().0, FieldError::ActionInvalid);
    }

    #[test]
    fn protocol_parse_err() {
        let s: &str = "deny [failhere] inside any outside any";
        assert_eq!(
            Rule::from_str(s).unwrap_err().0,
            FieldError::ProtocolUnsupported
        );
    }

    #[test]
    fn src_port_invalid() {
        let s: &str = "deny ip inside [failhere] outside any";
        assert_eq!(Rule::from_str(s).unwrap_err().0, FieldError::PortInvalid);
    }

    #[test]
    fn dst_port_invalid() {
        let s: &str = "deny ip inside any outside [failhere]";
        assert_eq!(Rule::from_str(s).unwrap_err().0, FieldError::PortInvalid);
    }
}
