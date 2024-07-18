#[allow(dead_code)]
mod acl;

#[allow(dead_code)]
mod junos;

use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum Action {
    Allow,
    Deny,
    AllowLog,
    DenyLog,
}

#[derive(Debug, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    IP,
}

#[derive(Debug, PartialEq)]
enum PortVariant {
    Any,
    Range(String),
    List(String),
    Num(u16),
}

#[derive(Debug, PartialEq)]
struct Rule {
    action: Action,
    protocol: Protocol,
    src_prefix: String,
    src_port: PortVariant,
    dst_prefix: String,
    dst_port: PortVariant,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <input_yaml>", args[0]);
    }

    let content = match fs::read_to_string(PathBuf::from(&args[1])) {
        Ok(str) => str,
        Err(e) => panic!("{}", e),
    };
    let configuration: acl::RulesetConfig = match serde_yml::from_str(&content) {
        Ok(ruleset) => ruleset,
        Err(e) => panic!("Error deserializing YAML: {}", e),
    };
    // println!("{:#?}", configuration);

    let validated_generics: Vec<Rule> = extrapolate_generics(&configuration.ruleset.generic);
    println!("{:#?}", validated_generics);

    println!("\nChecking interfaces assignments for ingress...");
    match check_ifaces_are_valid(&configuration.ruleset.deployment.ingress.interfaces) {
        Ok(_) => println!("Valid interface assignments for ingress."),
        Err(e) => panic!("{}", e),
    }

    println!("\nChecking interfaces assignments for egress...");
    match check_ifaces_are_valid(&configuration.ruleset.deployment.egress.interfaces) {
        Ok(_) => println!("Valid port assignments for egress."),
        Err(e) => panic!("{}", e),
    }
}

fn extrapolate_generics(generics: &Vec<String>) -> Vec<Rule> {
    let mut extrapolated_rules: Vec<Rule> = Vec::new();

    for rule in generics {
        let parts: Vec<&str> = rule.split_whitespace().collect();
        if parts.len() < 5 {
            panic!(
                "Failed to parse provided rule: {} :: Expected 5 fields, got {}",
                rule,
                parts.len()
            );
        }

        let extrapolated_rule: Rule = Rule {
            action: match parts[0] {
                "allow" => Action::Allow,
                "deny" => Action::Deny,
                "allowlog" => Action::AllowLog,
                "denylog" => Action::DenyLog,
                _ => panic!("Failed to parse action in rule: {} :: Field 0", rule),
            },
            protocol: match parts[1] {
                "tcp" => Protocol::TCP,
                "udp" => Protocol::UDP,
                "icmp" => Protocol::ICMP,
                "ip" => Protocol::IP,
                _ => panic!("Failed to parse action in rule: {} :: Field 1", rule),
            },
            src_prefix: parts[2].to_string(),
            src_port: match parse_portvariant(&parts[3]) {
                Ok(portvariant) => portvariant,
                Err(_) => panic!("Failed to parse src_port in rule: {} :: Field 3", rule),
            },
            dst_prefix: parts[4].to_string(),
            dst_port: match parse_portvariant(&parts[5]) {
                Ok(portvariant) => portvariant,
                Err(_) => panic!("Failed to parse src_port in rule: {} :: Field 5", rule),
            },
        };

        extrapolated_rules.push(extrapolated_rule);
    }

    extrapolated_rules
}

fn parse_portvariant(port: &str) -> Result<PortVariant, String> {
    match port.parse::<u16>() {
        Ok(num) => Result::Ok(PortVariant::Num(num)),
        Err(_e) => {
            if port.eq_ignore_ascii_case("any") {
                Result::Ok(PortVariant::Any)
            } else if port.contains("-") {
                Result::Ok(PortVariant::Range(port.to_string()))
            } else if port.contains(",") {
                Result::Ok(PortVariant::List(port.to_string()))
            } else {
                Result::Err(String::from("Division by zero"))
            }
        }
    }
}

fn check_ifaces_are_valid(interfaces: &Vec<String>) -> Result<(), String> {
    for iface in interfaces {
        // bad. this does not allow for branching into other supported device types
        match junos::is_valid_iface(&iface, junos::Srx1500::new().valid_ifaces) {
            Ok((true, str)) => println!(" - \'{}\' matched \'{}\'", iface, str),
            Ok((false, _str)) => {
                return Err(format!("InvalidPortAssigned on interface \"{}\"", iface));
            }
            Err(e) => panic!("{}", e),
        }
    }

    Ok(())
}
