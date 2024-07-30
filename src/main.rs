mod config;
mod device;
mod junos;
mod log;
mod ruleset;

use config::Configuration;
use log::LogLevel;
use ruleset::Ruleset;
use std::env;

pub const HELP_MSG: &str = r#"
Usage: am3k <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for additional logging and diagnostic information.
  -h, --help        Print this help message and exit.

Arguments:
  <config>          Path to the yaml configuration file.

Examples:
  am3k config.yaml
  am3k config.yaml -d

Description:
  ACL Manager 3000 (am3k) is used to build and manage access control lists via provided configuration file.
  The configuration file should be a YAML file specifying the rules and settings for the ACLs.

  - The <config> argument is mandatory and specifies the path to the configuration file.
  - Use the debug option to enable debug mode, which provides additional output useful in troubleshooting.

Notes:
  - Ensure the configuration file is correctly formatted as a YAML file.
  - The tool will output the resulting ACL to the standard output or to a specified file as configured.

For more information, visit: [[ NotYetImplementedError ]]

"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || ["-h", "--help"].contains(&args[1].as_str()) {
        println!("{}", HELP_MSG);
        return;
    }

    let mut dbg = LogLevel::Info;
    if args.len() == 3 && args[2].contains("d") {
        dbg = LogLevel::Debug;
        println!("Debug mode is enabled.");
    }

    let cfg: Configuration = Configuration::new(&args[1], dbg).unwrap();

    // CHECK DEVICES ARE SUPPORTED

    info!(dbg, "\nChecking device is supported...");

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
    dbug!(dbg, "{}", deployable_device);
    info!(dbg, "Platform and model are supported.");

    info!(dbg, "\nChecking interfaces assignments for ingress...");
    match deployable_device.are_valid_ifaces(&cfg.deployment.ingress.interfaces) {
        Ok(ifaces) => {
            dbug!(dbg, "{:#?}", ifaces);
            info!(dbg, "Valid interface assignments for ingress.");
        }
        Err(e) => {
            eprintln!("{:#?}", e);
            std::process::exit(1)
        }
    }

    info!(dbg, "\nChecking interfaces assignments for egress...");
    match deployable_device.are_valid_ifaces(&cfg.deployment.egress.interfaces) {
        Ok(ifaces) => {
            dbug!(dbg, "{:#?}", ifaces);
            info!(dbg, "Valid interface assignments for egress.");
        }
        Err(e) => {
            eprintln!("{:#?}", e);
            std::process::exit(1)
        }
    }

    info!(dbg, "\nChecking all rules are valid...");

    dbug!(dbg, "{:#?}", &cfg.ruleset);
    let validated_rules: Ruleset = match Ruleset::from_vec(&cfg.ruleset) {
        Ok(rules) => rules,
        Err(rule_errors) => {
            for (err, loc) in &rule_errors {
                eprintln!("{}:{} :: {}", &args[1], loc.line + 2, err);
            }
            eprintln!(
                "[ERROR]: {} configuration issues found while parsing rules.",
                rule_errors.len()
            );
            std::process::exit(1)
        }
    };
    dbug!(dbg, "{:#?}", &validated_rules);
    info!(dbg, "Valid rules provided in rules.");
}
