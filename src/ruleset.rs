#[allow(dead_code)]
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Action {
    Allow,
    Deny,
    AllowLog,
    DenyLog,
}

impl Action {
    const ERROR_MSG: &'static str = "ActionParseErr";
    const HELP_MSG: &'static str = "expected 'allow', 'deny', 'allowlog', or 'denylog'";
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

// Define Protocol enum
#[derive(Debug, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    IP,
}

impl Protocol {
    const ERROR_MSG: &'static str = "ProtocolParseErr";
    const HELP_MSG: &'static str = "expected 'ip', 'tcp', 'udp', or 'icmp'";
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

// Define PortVariant enum
#[derive(Debug, PartialEq)]
pub enum PortVariant {
    Any,
    Range(String),
    List(String),
    Num(u16),
}

impl PortVariant {
    const ERROR_MSG: &'static str = "PortInvalid";
    const HELP_MSG: &'static str =
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
                } else if s.contains("-") {
                    Ok(PortVariant::Range(s.to_string()))
                } else if s.contains(",") {
                    Ok(PortVariant::List(s.to_string()))
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

#[derive(Debug, PartialEq)]
pub struct Rule {
    action: Action,
    protocol: Protocol,
    src_prefix: String,
    src_port: PortVariant,
    dst_prefix: String,
    dst_port: PortVariant,
}

impl Rule {
    const ERROR_MSG: &'static str = "RuleLengthErr";
    const HELP_MSG: &'static str = "expected 6 fields";

    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(format!(
                "{}: {}, got {}",
                Rule::ERROR_MSG,
                Rule::HELP_MSG,
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

#[cfg(test)]
mod tests {
    use crate::ruleset::{Action, PortVariant, Protocol, Rule};

    #[test]
    fn rule_length_err_short() {
        let ss: &str = "short rule.";
        let result: String = Rule::from_str(ss).unwrap_err();
        let expected = format!("{}: {}, got 2", Rule::ERROR_MSG, Rule::HELP_MSG);
        assert_eq!(result, expected);
    }

    #[test]
    fn rule_length_err_long() {
        let ls: &str = "this is an extra long rule, ok.";
        let result: String = Rule::from_str(ls).unwrap_err();
        let expected = format!("{}: {}, got 7", Rule::ERROR_MSG, Rule::HELP_MSG);
        assert_eq!(result, expected);
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
