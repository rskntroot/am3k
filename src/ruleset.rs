#[allow(dead_code)]
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
enum Action {
    Allow,
    Deny,
    AllowLog,
    DenyLog,
}

impl Action {
    pub const ERROR_MSG: &'static str = "ActionParseErr";
    pub const HELP_MSG: &'static str = "expected 'allow', 'deny', 'allowlog', or 'denylog'";
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allow" => Ok(Action::Allow),
            "deny" => Ok(Action::Deny),
            "allowlog" => Ok(Action::AllowLog),
            "denylog" => Ok(Action::DenyLog),
            _ => Err(format!("{}: {}", Action::ERROR_MSG, Action::HELP_MSG)),
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

impl Protocol {
    pub const ERROR_MSG: &'static str = "ProtocolParseErr";
    pub const HELP_MSG: &'static str = "expected 'ip', 'tcp', 'udp', or 'icmp'";
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::TCP),
            "udp" => Ok(Protocol::UDP),
            "icmp" => Ok(Protocol::ICMP),
            "ip" => Ok(Protocol::IP),
            _ => Err(format!("{}: {}", Protocol::ERROR_MSG, Protocol::HELP_MSG)),
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
pub enum PortVariant {
    Any,
    Range(String),
    List(String),
    Num(u16),
}

impl PortVariant {
    pub const ERROR_MSG: &'static str = "PortInvalid";
    pub const HELP_MSG: &'static str =
        "expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'";
}

impl FromStr for PortVariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(num) => Result::Ok(PortVariant::Num(num)),
            Err(_) => {
                if s == "any" {
                    Ok(PortVariant::Any)
                } else if s.contains(",") {
                    Ok(PortVariant::List(s.to_string()))
                } else if s.matches('-').count() == 1 {
                    Ok(PortVariant::Range(s.to_string()))
                } else {
                    Err(format!(
                        "{}: {}",
                        PortVariant::ERROR_MSG,
                        PortVariant::HELP_MSG
                    ))
                }
            }
        }
    }
}

impl fmt::Display for PortVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PortVariant::Any => write!(f, "any"),
            PortVariant::Range(range) => write!(f, "{}", range),
            PortVariant::List(list) => write!(f, "{}", list),
            PortVariant::Num(num) => write!(f, "{}", num),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    action: Action,
    protocol: Protocol,
    src_prefix: String,
    src_port: PortVariant,
    dst_prefix: String,
    dst_port: PortVariant,
}

impl Rule {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(format!(
                "{}: {}, got {}",
                "RuleLengthErr",
                "expected 6 fields",
                parts.len(),
            ));
        }

        if parts[3].contains(',') && parts[5].contains(',') {
            return Err(format!(
                "{}: {}, got {}",
                "PortExpansionUnsupported",
                "both src & dst ports cannot be port lists",
                parts.len(),
            ));
        }

        Ok(Rule {
            action: match Action::from_str(parts[0]) {
                Ok(action) => action,
                Err(e) => return Err(e),
            },
            protocol: match Protocol::from_str(parts[1]) {
                Ok(protocol) => protocol,
                Err(e) => return Err(e),
            },
            src_prefix: parts[2].to_string(),
            src_port: match PortVariant::from_str(parts[3]) {
                Ok(portvariant) => portvariant,
                Err(e) => return Err(String::from("Src") + &e),
            },
            dst_prefix: parts[4].to_string(),
            dst_port: match PortVariant::from_str(parts[5]) {
                Ok(portvariant) => portvariant,
                Err(e) => return Err(String::from("Dst") + &e),
            },
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

#[derive(Debug)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
}

impl Ruleset {
    pub fn new(raw_rules: &Vec<String>) -> Result<Self, Vec<(String, String, usize)>> {
        let rules: Vec<Rule> = match parse_rules(raw_rules) {
            Ok(rs) => rs,
            Err(e) => return Err(e),
        };

        let rules: Vec<Rule> = match expand_rules(rules) {
            Ok(rs) => rs,
            Err(e) => return Err(e),
        };

        Ok(Ruleset { rules })
    }
}

/// ## Function
/// parses rules from raw strings to validated rules that may require expansion
/// ## Logic
/// - if all rules are valid Returns `Ok(Vec<crate::ruleset::Rule>)`
/// - else returns list a rules with errors Returns: `Err(Vec<(String, String, usize)>)`
///     - as `(<error>, <rule>, <index>)`
fn parse_rules(raw_rules: &Vec<String>) -> Result<Vec<Rule>, Vec<(String, String, usize)>> {
    let mut valid_rules: Vec<Rule> = Vec::new();
    let mut errors: Vec<(String, String, usize)> = Vec::new();

    for (i, rule) in raw_rules.iter().enumerate() {
        valid_rules.push(match Rule::from_str(rule) {
            Ok(r) => r,
            Err(e) => {
                errors.push((String::from(e), String::from(rule), i));
                continue;
            }
        });
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(valid_rules)
}

/// ## Function
/// expands all ranges of rules into individual entries to support
/// deployment tools that do not have builtin handling for ranges.
/// ## Logic
/// - check each rule for src or dst for list of ports
/// - expand all rules that are lists
/// - if all rules are valid Returns `Ok(Vec<crate::ruleset::Rule>)`
/// - else returns list a rules with errors Returns: `Err(Vec<(String, String, usize)>)`
///     - as `(<error>, <rule>, <index>)`
fn expand_rules(rule_list: Vec<Rule>) -> Result<Vec<Rule>, Vec<(String, String, usize)>> {
    let mut new_rule_list: Vec<Rule> = vec![];
    let mut errors: Vec<(String, String, usize)> = Vec::new();

    for (i, rule) in rule_list.iter().enumerate() {
        if let PortVariant::List(_) = rule.src_port {
            for new_port in expand_port_list(&rule.src_port).unwrap() {
                let mut base_rule: Rule = rule.clone();
                base_rule.src_port = match PortVariant::from_str(&new_port) {
                    Ok(port) => port,
                    Err(e) => {
                        errors.push((format!("ExpandingSrc{}", e), format!("{}", &rule), i));
                        continue;
                    }
                };

                new_rule_list.push(base_rule);
            }
        } else if let PortVariant::List(_) = rule.dst_port {
            for new_port in expand_port_list(&rule.dst_port).unwrap() {
                let mut base_rule: Rule = rule.clone();
                base_rule.dst_port = match PortVariant::from_str(&new_port) {
                    Ok(port) => port,
                    Err(e) => {
                        errors.push((format!("ExpandingDst{}", e), format!("{}", &rule), i));
                        continue;
                    }
                };

                new_rule_list.push(base_rule);
            }
        } else {
            new_rule_list.push(rule.clone());
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(new_rule_list)
}


/// shorthand port string split on comma
fn expand_port_list(list: &PortVariant) -> Result<Vec<String>, String> {
    match list {
        PortVariant::List(_) => {}
        _ => return Err(format!("parse_port_list() only takes PortVariant::List")),
    }

    Ok(list
        .to_string()
        .split(',')
        .map(|i| String::from(i))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::ruleset::{Action, PortVariant, Protocol, Rule, Ruleset};
    #[test]
    fn portlist_expansion_valid() {
        let rs: Vec<String> = vec![
            "allow udp outside any inside 161,162".to_string(),
            "allow tcp inside any outside 22,80,443,9000-9010".to_string(),
        ];
        dbg!(Ruleset::new(&rs).unwrap());
    }

    #[test]
    fn portlist_expansion_invalid() {
        let rs: Vec<String> = vec![
            "allow udp outside any inside 161,,162".to_string(),
            "allow tcp inside 22,*,443,9000-9010 outside any".to_string(),
        ];
        dbg!(Ruleset::new(&rs).unwrap_err());
    }

    #[test]
    fn rule_contains_multiple_lists() {
        let rs: Vec<String> = vec![
            "allow tcp inside 20,21 outside 9000,9010".to_string(),
        ];
        dbg!(Ruleset::new(&rs).unwrap_err());
    }

    #[test]
    fn rule_length_err_short() {
        let ss: &str = "short rule.";
        let result: String = Rule::from_str(ss).unwrap_err();
        assert!(result.contains("RuleLengthErr"), "unexpected error thrown");
        assert!(result.contains("got 2"), "unexpected rule length");
    }

    #[test]
    fn rule_length_err_long() {
        let ls: &str = "this is an extra long rule, ok.";
        let result: String = Rule::from_str(ls).unwrap_err();
        assert!(result.contains("RuleLengthErr"), "unexpected error thrown");
        assert!(result.contains("got 7"), "unexpected rule length");
    }

    #[test]
    fn action_parse_err() {
        let s: &str = "[failhere] ip inside any outside any";
        let result: String = Rule::from_str(s).unwrap_err();
        println!("{}", result);
        assert!(result.contains(Action::ERROR_MSG), "error_msg missing");
        assert!(result.contains(Action::HELP_MSG), "help_msg missing");
    }

    #[test]
    fn protocol_parse_err() {
        let s: &str = "deny [failhere] inside any outside any";
        let result: String = Rule::from_str(s).unwrap_err();
        assert!(result.contains(Protocol::ERROR_MSG), "error_msg missing");
        assert!(result.contains(Protocol::HELP_MSG), "help_msg missing");
    }

    #[test]
    fn src_port_invalid() {
        let s: &str = "deny ip inside [failhere] outside any";
        let result: String = Rule::from_str(s).unwrap_err();
        assert!(result.contains(PortVariant::ERROR_MSG), "error_msg missing");
        assert!(result.contains(PortVariant::HELP_MSG), "error_msg missing");
    }

    #[test]
    fn dst_port_invalid() {
        let s: &str = "deny ip inside any outside failhere";
        let result: String = Rule::from_str(s).unwrap_err();
        assert!(result.contains(PortVariant::ERROR_MSG), "error_msg missing");
        assert!(result.contains(PortVariant::HELP_MSG), "error_msg missing");
    }
}
