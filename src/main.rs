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
use tera::Tera;

pub const HELP_MSG: &str = r#"
Usage: am3k <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for all logging and diagnostic information.
  -h, --help        Print this help message and exit.
  -v, --verbose     Enable verbose mode for additional logging and diagnostic information.

Arguments:
  <config>          Path to the yaml configuration file.

Environment:
  AM3K_PLATFORMS_PATH     Path to the directory containing platform definitions. Defaults to "./platform".
  AM3K_ACL_PATH           Path to the directory containing ACL definitions. Defaults to "./acls".

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

    let acls_path: String = match std::env::var("AM3K_PLATFORMS_PATH") {
        Ok(path) => path.trim_end_matches('/').to_string(),
        Err(_) => String::from("./acls"),
    };

    // configuration is mandatory
    info!(dbg, "\nLoading configuration file {}...", &args[1]);
    let cfg: Configuration = match Configuration::new(&args[1], &acls_path, dbg) {
        Ok(Some(config)) => config,
        Err(e) => {
            crit!(dbg, "{}", e);
            std::process::exit(1)
        }
        Ok(None) => {
            crit!(dbg, "{}", config::ConfigInvalid::FailedPostChecks);
            std::process::exit(2)
        }
    };
    info!(dbg, "Configuration file loaded successfully from yaml.");

    let mut buildable: bool = true;

    // build an optional device
    info!(dbg, "\nChecking platform is supported...");
    let _deployable_device: Option<Device> = match Device::build(
        "model-citizen",
        &cfg.deployment.platform.make,
        &cfg.deployment.platform.model,
        &cfg.deployment.ingress.interfaces,
        &cfg.deployment.egress.interfaces,
        dbg,
    ) {
        Ok(device) => Some(device),
        Err(e) => {
            crit!(dbg, "{}", e);
            buildable = false;
            None
        }
    };
    match buildable {
        true => info!(dbg, "Platform is supported."),
        false => info!(dbg, "Platform is not supported."),
    }

    // build a vec of optional rulesets
    info!(dbg, "\nLoading rulesets...");
    dbug!(dbg, "{:#?}", &cfg.deployment.rulesets);
    let mut validated_rulesets: Vec<Option<Ruleset>> = vec![];
    for ruleset in &cfg.deployment.rulesets {
        let acls_path = format!("{}/{}.acl", &acls_path, ruleset);
        match Ruleset::new(&acls_path, dbg) {
            Ok(ruleset) => {
                verb!(dbg, "{}", &ruleset.to_string());
                validated_rulesets.push(Some(ruleset))
            }
            Err(e) => {
                crit!(dbg, "* Ruleset issues found while parsing:\n{}", e);
                buildable = false;
                validated_rulesets.push(None);
            }
        }
    }
    match buildable {
        true => info!(dbg, "Valid rules provided in rulesets."),
        false => info!(dbg, "Invalid rules provided in rulesets."),
    }

    if !buildable {
        crit!(
            dbg,
            "Unable to generate output with provided configuration and rulesets."
        );
        std::process::exit(3);
    }

    verb!(dbg, "\nPacking Tera context...");
    let mut context = tera::Context::new();
    context.insert("rulesets", &contextualize(&validated_rulesets).unwrap());
    context.insert("device", &contextualize(&_deployable_device).unwrap());
    context.insert("config", &contextualize(&cfg).unwrap());
    if dbg.value() <= LogLevel::Debug.value() {
        dbg!(&context);
    }
    verb!(dbg, "Packing succeeded.");

    let tera = match Tera::new("tmpl/**/*") {
        Ok(t) => t,
        Err(e) => {
            crit!(dbg, "{}", e);
            std::process::exit(4)
        }
    };

    // output rendered tera using tmpl/ruleset.tera
    let rendered = match tera.render("ruleset.tera", &context) {
        Ok(render) => render,
        Err(e) => {
            crit!(dbg, "{}", e);
            std::process::exit(5)
        }
    };

    info!(dbg, "\n{}", rendered);
}
