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
    const HELP_MSG: &'static str = "expected 'allow', 'deny', 'allowlog', or 'denylog'";
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
                    Err(String::from(
                        "PortInvalid :: expected a port, range list, or 'any'",
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
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() < 5 {
            return Err(format!(
                "RuleLengthErr :: Expected 5 fields, got {}",
                parts.len()
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
