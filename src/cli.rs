use std::fmt;

use crate::LogLevel;
use clap::{Arg, ArgAction, ArgGroup, Command};

#[derive(Debug)]
pub struct Args {
    pub config: String,
    pub loglevel: LogLevel,
    pub env: EnvVars,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "config: {}, loglevel: {}, env: {}",
            self.config, self.loglevel, self.env
        )
    }
}

#[derive(Debug)]
pub struct EnvVars {
    pub platforms: String,
    pub rulesets: String,
    pub templates: String,
}

impl fmt::Display for EnvVars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "platforms: {}, rulesets: {}, templates: {}",
            self.platforms, self.rulesets, self.templates
        )
    }
}

/// loads `command line arguments` and `environment variables` using custom `clap`
pub fn parse_args() -> Args {
    let matches: clap::ArgMatches = build().get_matches();

    let config: String = matches.get_one::<String>("config").unwrap().to_string();

    let loglevel: LogLevel = match matches.get_flag("debug") {
        true => LogLevel::Debug,
        false => match matches.get_flag("verbose") {
            true => LogLevel::Verbose,
            false => LogLevel::Info,
        },
    };

    let env: EnvVars = parse_env();

    Args {
        config,
        loglevel,
        env,
    }
}

/// loads only environment variables
pub fn parse_env() -> EnvVars {
    EnvVars {
        platforms: match std::env::var("AM3K_PLATFORMS_PATH") {
            Ok(path) => path.trim_end_matches('/').to_string(),
            Err(_) => String::from("./platform"),
        },
        rulesets: match std::env::var("AM3K_RULESET_PATH") {
            Ok(path) => path.trim_end_matches('/').to_string(),
            Err(_) => String::from("./acls"),
        },
        templates: match std::env::var("AM3K_TEMPLATES_PATH") {
            Ok(path) => path.trim_end_matches('/').to_string(),
            Err(_) => String::from("./tmpl"),
        },
    }
}

const ABOUT_MSG: &str = r#"(am3k) Access Control List Manager 3000"#;
const ENV_MSG: &str = r#"Environment:
    AM3K_PLATFORMS_PATH     Path to the directory containing platform definitions. Defaults to "./platform".
    AM3K_RULESETS_PATH      Path to the directory containing ACL definitions. Defaults to "./acls".
    AM3K_TEMPLATES_PATH     Path to the directory containing template definitions. Defaults to "./tmpl".
"#;

/// builds a custom command line argument parser
fn build() -> Command {
    Command::new("app")
        .about(ABOUT_MSG)
        .version(env!("CARGO_PKG_VERSION"))
        .author("rskntroot")
        .arg(
            Arg::new("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .required(true),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Print debug information")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print verbose information")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .group(
            ArgGroup::new("loglevel")
                .args(&["debug", "verbose"])
                .required(false),
        )
        .after_help(ENV_MSG)
}
