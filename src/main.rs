#[allow(dead_code)]
mod config;
mod ruleset;
use crate::ruleset::Rule;
#[allow(dead_code)]
mod device;
mod junos;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <input_yaml>", args[0]);
    }

    let content = match fs::read_to_string(PathBuf::from(&args[1])) {
        Ok(str) => str,
        Err(e) => panic!("{}", e),
    };
    let configuration: config::RulesetConfig = match serde_yml::from_str(&content) {
        Ok(ruleset) => ruleset,
        Err(e) => panic!("Error deserializing YAML: {}", e),
    };
    // println!("{:#?}", configuration);

    println!("\nChecking all generics are valid rules...");
    let validated_generics: Vec<Rule> = match parse_generic_rules(&configuration.ruleset.generic) {
        Ok(rules) => rules,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}", e);
            }
            panic!(
                "GenericsRuleParser found {} errors. Please update rules.",
                errors.len()
            );
        }
    };
    println!("{:#?}", validated_generics);
    println!("Valid rules provided in generics.");

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

fn parse_generic_rules(rules: &Vec<String>) -> Result<Vec<Rule>, Vec<String>> {
    let mut parsed_rules: Vec<Rule> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for rule in rules {
        parsed_rules.push(match Rule::from_str(rule) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("\n{} on\n  - {}", e, rule).to_string());
                continue;
            }
        });
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(parsed_rules)
}

// todo add multi-device support
fn check_ifaces_are_valid(interfaces: &Vec<String>) -> Result<(), String> {
    for iface in interfaces {
        match junos::srx1500("example").is_valid_iface(&iface) {
            Ok((true, str)) => println!(" - \'{}\' matched \'{}\'", iface, str),
            Ok((false, _str)) => {
                return Err(format!("InvalidPortAssigned on interface \"{}\"", iface));
            }
            Err(e) => panic!("{}", e),
        }
    }

    Ok(())
}
