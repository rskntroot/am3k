use serde::Deserialize;

// ============== RULESET CONFIGURATIION FILE ==============

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

// test struct is constructable
#[test]
fn test_acl_config() {
    let test_data = r#"
ruleset: 
  generic:
    - allow icmp outside any inside 8
    - deny tcp outside any inside 22
    - allowlog ip outside any inside 80,443
    - denylog udp outside any inside 161-162
  deployment:
    ingress:
      devicelist: [ "test-device" ]
      interfaces: [ "ae10" ]
      filters:
        src: [ "example" ]
        dst: [ "example" ]
      deployable: True
      established: True
      default: deny
      transforms:
        src: False
        dst: False
    egress:
      devicelist:
        - test-device
      interfaces:
        - ae20
      filters:
        src: 
          - "example"
        dst:
          - "example"
      deployable: True
      established: True
      default: deny
      transforms:
        src: False
        dst: False
    "#;

    let result: Result<Ruleset, serde_yml::Error> = serde_yml::from_str(test_data);

    let _configuration: Ruleset = match result {
        Ok(ruleset) => ruleset,
        Err(_) => {
            panic!("Error deserializing YAML");
        }
    };
}
