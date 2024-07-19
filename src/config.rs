use serde::Deserialize;

#[derive(Debug, Deserialize)]

pub struct RulesetConfig {
    pub ruleset: Ruleset,
}

#[derive(Debug, Deserialize)]
pub struct Ruleset {
    pub generic: Vec<String>,
    pub deployment: DeploymentRules,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentRules {
    pub platform: String,
    pub model: String,
    pub ingress: Ingress,
    pub egress: Egress,
}

#[derive(Debug, Deserialize)]
pub struct Ingress {
    pub devicelist: Vec<String>,
    pub interfaces: Vec<String>,
    pub filters: Filters,
    pub deployable: bool,
    pub established: bool,
    pub default: String,
    pub transforms: Transforms,
}

#[derive(Debug, Deserialize)]
pub struct Egress {
    pub devicelist: Vec<String>,
    pub interfaces: Vec<String>,
    pub filters: Filters,
    pub deployable: bool,
    pub established: bool,
    pub default: String,
    pub transforms: Transforms,
}

#[derive(Debug, Deserialize)]
pub struct Filters {
    pub src: Vec<String>,
    pub dst: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Transforms {
    pub src: bool,
    pub dst: bool,
}
