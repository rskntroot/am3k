mod config;
mod device;
mod junos;
mod ruleset;

use crate::ruleset::Ruleset;

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

    // CHECK DEVICE NAMES ARE VALID

    check_index += 1;
    println!(
        "\n{}. Checking devicelist against device naming convention...",
        check_index
    );

    match check_device_names(&cfg.deployment.devicelist, &cfg.defaults.device_regex) {
        Ok(_) => println!("Valid device names per naming convention"),
        Err(e) => panic!("{:#?} on {}", e, cfg.defaults.device_regex),
    };

    // CHECK DEVICES ARE SUPPORTED

    check_index += 1;
    println!(
        "\n{}. Checking platform and model are supported...",
        check_index
    );

    let deployable_device = match cfg.deployment.platform.as_str() {
        "junos" => match junos::new("generic-device", &cfg.deployment.model) {
            Ok(d) => d,
            Err(e) => panic!("{}", e),
        },
        _ => panic!(
            "{}: {}",
            device::SupportedPlatform::ERROR_MSG,
            device::SupportedPlatform::HELP_MSG,
        ),
    };
    user_dbg!(debug_mode, "{}", deployable_device);
    println!("Platform and model are supported.");

    // CHECK INTERFACE ASSIGNMENTS ARE VALID

    check_index += 1;
    println!(
        "\n{}. Checking interfaces assignments for ingress...",
        check_index
    );

    match deployable_device.are_valid_ifaces(&cfg.deployment.ingress.interfaces) {
        Ok(ifaces) => {
            user_dbg!(debug_mode, "{:#?}", ifaces);
            println!("Valid interface assignments for ingress.");
        }
        Err(e) => panic!("{:#?}", e),
    }

    check_index += 1;
    println!(
        "\n{}. Checking interfaces assignments for egress...",
        check_index
    );

    match deployable_device.are_valid_ifaces(&cfg.deployment.ingress.interfaces) {
        Ok(ifaces) => {
            user_dbg!(debug_mode, "{:#?}", ifaces);
            println!("Valid interface assignments for egress.");
        }
        Err(e) => panic!("{:#?}", e),
    }

    // CHECK RULES ARE VALID

    check_index += 1;
    println!("\n{}. Checking all rules are valid...", check_index);

    user_dbg!(debug_mode, "{:#?}", &cfg.ruleset);
    let validated_rules: Ruleset = match Ruleset::new(&cfg.ruleset) {
        Ok(rules) => rules,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}:{} :: {}", &args[1], e.2 + 2, e.0);
            }
            panic!(
                " - RulesParser found {} errors. Please update rules.",
                errors.len()
            );
        }
    };
    user_dbg!(debug_mode, "{:#?}", &validated_rules.rules);
    println!("Valid rules provided in rules.");
}

/// ## Function
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
            _ => errors.push(format!("DeviceNameInvalid on device {}", &device)),
        };
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn device_has_valid_name() {
        let device_name: String = String::from("rsk101-example-fw1");
        let pattern: &str =
            "^[a-z]{1,3}([0-9]{1,10}-){1,2}([a-z]{2,9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0,9})?";

        let result: bool = super::check_device_names(&vec![device_name], pattern).unwrap();

        assert_eq!(result, true);
    }

    #[test]
    fn device_has_invalid_name() {
        let device_name: String = String::from("firewall1");
        let pattern: &str =
            "^[a-z]{1,3}([0-9]{1,10}-){1,2}([a-z]{2,9}-){1,4}[a-z]{1,5}[1-9]([0-9]{0,9})?";

        let result: Vec<String> =
            super::check_device_names(&vec![device_name], pattern).unwrap_err();

        assert!(
            result[0].contains("DeviceNameInvalid"),
            "Device unexpectedly matched regex."
        );
    }
}
