mod config;
mod device;
mod log;
mod ruleset;

use config::Configuration;
use device::Device;
use log::LogLevel;
use ruleset::Ruleset;
use serde_json::to_value as contextualize;
use std::env;
// use tera::Context; // NotYetImpld
// use tera::Tera; // NotYetImpld

pub const HELP_MSG: &str = r#"
Usage: am3k <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for all logging and diagnostic information.
  -h, --help        Print this help message and exit.
  -v, --verbose     Enable verbose mode for additional logging and diagnostic information.

Arguments:
  <config>          Path to the yaml configuration file.

Environment:
  AM3K_PLATFORMS_PATH     Path to the directory containing platform definitions. Defaults to "./platform/".

Examples:
  am3k config.yaml
  am3k config.yaml -d

Description:
  ACL Manager 3000 (am3k) is used to build and manage access control lists via provided configuration file.
  The configuration file should be a YAML file specifying the rules and settings for the ACLs.

For more information, visit: [NotYetImpld]

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

    let cfg: Configuration = match Configuration::new(&args[1], dbg) {
        Ok(config) => config,
        Err(e) => {
            crit!(dbg, "{}", e);
            std::process::exit(1)
        }
    };

    info!(dbg, "\nChecking platform is supported...");
    let _deployable_device: Device = match Device::build(
        "model-citizen",
        &cfg.deployment.platform.make,
        &cfg.deployment.platform.model,
        &cfg.deployment.ingress.interfaces,
        &cfg.deployment.egress.interfaces,
        dbg,
    ) {
        Ok(device) => device,
        Err(e) => {
            crit!(dbg, "{}", e);
            std::process::exit(1)
        }
    };
    info!(dbg, "Platform is supported.");

    info!(dbg, "\nChecking all rules are valid...");
    dbug!(dbg, "{:#?}", &cfg.ruleset);
    let validated_rules: Ruleset = match Ruleset::from_vec(&cfg.ruleset) {
        Ok(rules) => rules,
        Err(rule_errors) => {
            for (err, loc) in &rule_errors {
                crit!(
                    dbg,
                    "{}:{}:{}\t{}",
                    &args[1],
                    loc.line + 2,
                    loc.column + 5,
                    err
                );
            }
            crit!(
                dbg,
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
    info!(dbg, "{}", &expanded_rules);
    info!(dbg, "Ruleset expanded.");

    verb!(dbg, "\nPacking as JSON for Tera context...");
    let json: tera::Value = contextualize(&expanded_rules).unwrap();
    dbug!(dbg, "{}", serde_json::to_string_pretty(&json).unwrap());
    verb!(dbg, "Packing succeeded.");
}
