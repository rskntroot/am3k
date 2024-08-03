mod config;
mod device;
mod junos;
mod log;
mod ruleset;

use config::Configuration;
use log::LogLevel;
use ruleset::Ruleset;
use serde_json::to_value as contextualize;
use std::env;
// use tera::Context; // NotYetImpld
// use tera::Tera; // NotYetImpld

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
    if args.len() == 3 {
        if args[2].contains("d") {
            dbg = LogLevel::Debug;
            println!("Debug mode is enabled.");
        } else if args[2].contains("v") {
            dbg = LogLevel::Verbose;
            println!("Verbose mode is enabled.");
        }
    }

    let cfg: Configuration = Configuration::new(&args[1], dbg).unwrap();

    // todo address method for determining various devices.
    // via load supported devices file wtih valid regex at runtime
    info!(dbg, "\nChecking device is supported...");
    verb!(
        dbg,
        "attempting to create device based on platform: {} {}",
        &cfg.deployment.platform,
        &cfg.deployment.model
    );
    let deployable_device = match junos::new("model-citizen", &cfg.deployment.model) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
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
                eprintln!("{}:{}:{} {}", &args[1], loc.line + 2, loc.column + 5, err);
            }
            eprintln!(
                "Rule configuration issues found while parsing: {}",
                rule_errors.len()
            );
            std::process::exit(1)
        }
    };
    verb!(dbg, "{}", &validated_rules);
    info!(dbg, "Valid rules provided in rules.");

    info!(dbg, "\nExpanding ruleset...");
    let expanded_rules: Ruleset = validated_rules.expand();
    verb!(dbg, "{}", &expanded_rules);
    info!(dbg, "Ruleset expanded.");

    verb!(dbg, "\nPacking as JSON for Tera context...");
    let json: tera::Value = contextualize(&expanded_rules).unwrap();
    dbug!(dbg, "{}", serde_json::to_string_pretty(&json).unwrap());
    verb!(dbg, "Packing succeeded.");
}
