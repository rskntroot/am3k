mod config;
mod device;
mod junos;
mod ruleset;

use crate::config::DeploymentRules;
use crate::ruleset::Rule;

use std::env;
use std::fs;
use std::path::PathBuf;

use regex::Regex;

macro_rules! user_dbg {
    ($debug_mode:expr, $($msg:expr),*) => {
        if $debug_mode {
            println!($($msg),*);
        }
    };
}

pub const HELP_MSG: &str = r#"
Usage: acl-builder <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for additional logging and diagnostic information.
  -h, --help        Print this help message and exit.

Arguments:
  <config>          Path to the yaml configuration file.

Examples:
  acl-builder config.yaml
  acl-builder config.yaml --debug
  acl-builder config.yaml -d

Description:
  The acl-builder tool is used to build and manage access control lists (ACLs) based on the provided configuration file.
  The configuration file should be a YAML file specifying the rules and settings for the ACL.

  - The <config> argument is mandatory and specifies the path to the configuration file.
  - Use the -d or --debug option to enable debug mode, which provides additional output useful for debugging.

Notes:
  - Ensure the configuration file is correctly formatted as a YAML file.
  - The tool will output the resulting ACL to the standard output or to a specified file as configured.

For more information, visit: [[ NotYetImplementedError ]]

"#;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug_mode: bool = false;

    if args.len() < 2 || ["-h", "--help"].contains(&args[1].as_str()) {
        println!("{}", HELP_MSG);
        return;
    }

    if args.len() == 3 && args[2].contains("d") {
        debug_mode = true;
        user_dbg!(debug_mode, "Debug mode is enabled.");
    }

    println!("\nLoading configuration file {}...", &args[1]);
    let content = match fs::read_to_string(PathBuf::from(&args[1])) {
        Ok(str) => str,
        Err(e) => panic!("{}", e),
    };
    let cfg: config::Configuration = match serde_yml::from_str(&content) {
        Ok(ruleset) => ruleset,
        Err(e) => panic!("Error deserializing YAML: {}", e),
    };
    user_dbg!(debug_mode, "{:#?}", cfg);
    println!("Configuration file loaded without issue.");

    println!("\nChecking configuration file components are valid:");
    let mut check_index: u8 = 0;

    // CHECK DEVICES ARE SUPPORTED

    check_index += 1;
    println!(
        "\n{}. Checking platform and model are supported...",
        check_index
    );

    match check_device_supported(
        &cfg.ruleset.deployment.platform,
        &cfg.ruleset.deployment.model,
    ) {
        Ok(_) => println!(
            "Deployments for {} {} are supported.",
            &cfg.ruleset.deployment.platform, &cfg.ruleset.deployment.model
        ),
        Err(e) => panic!("{}", e),
    }

    let deployable_device = junos::srx1500("generic-device");
    let deployment: &DeploymentRules = &cfg.ruleset.deployment;

    // CHECK DEVICE NAMES ARE VALID

    check_index += 1;
    println!(
        "\n{}. Checking devicelist against device naming convention...",
        check_index
    );

    match check_device_names(&deployment.devicelist, &cfg.defaults.device_regex) {
        Ok(_) => println!("Valid device names per naming convention"),
        Err(e) => panic!("{:#?} on {}", e, cfg.defaults.device_regex),
    };

    // CHECK INTERFACE ASSIGNMENTS ARE VALID

    check_index += 1;
    println!(
        "\n{}. Checking interfaces assignments for ingress...",
        check_index
    );

    match check_ifaces_are_valid(&deployable_device, &deployment.ingress.interfaces) {
        Ok(ifaces) => {
            user_dbg!(debug_mode, "{:#?}", ifaces);
            println!("Valid interface assignments for ingress.");
        },
        Err(e) => panic!("{:#?}", e),
    }

    check_index += 1;
    println!(
        "\n{}. Checking interfaces assignments for egress...",
        check_index
    );

    match check_ifaces_are_valid(&deployable_device, &deployment.ingress.interfaces) {
        Ok(ifaces) => {
            user_dbg!(debug_mode, "{:#?}", ifaces);
            println!("Valid interface assignments for egress.");
        },
        Err(e) => panic!("{:#?}", e),
    }

    // CHECK GENERIC RULES ARE VALID

    check_index += 1;
    println!(
        "\n{}. Checking all generics are valid rules...",
        check_index
    );

    println!("{:#?}", &cfg.ruleset.generic);
    let validated_generics: Vec<Rule> = match get_valid_generic_rules(&cfg.ruleset.generic) {
        Ok(rules) => rules,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}", e);
            }
            panic!(
                " - GenericsRuleParser found {} errors. Please update rules.",
                errors.len()
            );
        }
    };
    user_dbg!(debug_mode, "{:#?}", validated_generics);
    println!("Valid rules provided in generics.");
}

fn check_device_supported(make: &str, model: &str) -> Result<bool, String> {
    match make {
        "junos" => match junos::SUPPORTED_DEVICES.contains(&model) {
            true => return Ok(true),
            _ => return Err(format!("{} model {} is not supported", make, model)),
        },
        _ => return Err(format!("Platform {} is not supported", make)),
    }
}

/// Regex lookup for all devices in provided `devicelist` against provided `pattern`
/// - if all devices in devicelist match the provided patternReturns `Result: Ok(true)`
/// - else returns list of devices with DeviceNameInvalid `Result: Err(Vec<String>)`
/// - note: does not return `false`
fn check_device_names(devicelist: &Vec<String>, pattern: &str) -> Result<bool, Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let pattern: Regex = match Regex::new(pattern) {
        Ok(exp) => exp,
        Err(_) => {
            errors.push(String::from("InvalidRegexExpression"));
            return Err(errors);
        }
    };

    for device in devicelist {
        match pattern.is_match(&device) {
            true => {}
            _ => errors.push(format!("DeviceNameInvalid on device {}", device)),
        };
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(true)
}

// TODO: add docstring
fn check_ifaces_are_valid(
    device: &device::Device,
    interfaces: &Vec<String>,
) -> Result<Vec<String>, Vec<String>> {
    let mut valid_ifaces: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for iface in interfaces {
        match device.is_valid_iface(&iface) {
            Ok((true, str)) => valid_ifaces.push(format!(" - \'{}\' matched \'{}\'", iface, str)),
            Ok((false, _str)) => {
                errors.push(format!("InvalidPortAssigned on interface \'{}\'", iface))
            }
            Err(e) => panic!("{}", e),
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(valid_ifaces)
}

// TODO: add docstring
fn get_valid_generic_rules(rules: &Vec<String>) -> Result<Vec<Rule>, Vec<String>> {
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
