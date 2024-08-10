mod cli;
mod config;
mod device;
mod log;
mod ruleset;

use config::Configuration;
use device::Device;
use log::LogLevel;
use ruleset::Ruleset;
use serde_json::to_value as contextualize;
use tera::Tera;

fn main() {
    let args: cli::Args = cli::parse_args();
    let dbg: LogLevel = args.loglevel;

    // configuration is mandatory
    info!(dbg, "\nLoading configuration file {}...", &args.config);
    let cfg: Configuration = match Configuration::load(&args.config, &args.env.rulesets, dbg) {
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
        &args.env.platforms,
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
        let acls_path = format!("{}/{}.acl", &args.env.rulesets, ruleset);
        match Ruleset::load(&acls_path, dbg) {
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
