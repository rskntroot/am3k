# am3k

acl manager 3000

## Brief

a work in progress acl creation kit

a project that mainly exists for me to learn more rust

## Examples

### Usage

```
$ target/release/am3k -h

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
```

### Example Output

```
$ target/release/am3k site/example.yaml -v
Verbose mode is enabled.

Loading configuration file site/example.yaml...
  Checking devicelist naming convention...
  Devices matched convention.

  Checking ruleset files exist...
  Ruleset files exist.
Configuration file loaded successfully from yaml.

Checking platform is supported...
  Loading path to supported platforms...
  Found path: ./platform

  Searching for matching supported platform file...
  Found ./platform/juniper.yaml

  Loading supported platforms file...
  Platforms file loaded successfully from yaml.

  Checking supported model...
  Model supported.

  Confirming interfaces are valid...
  Interfaces are valid
Platform is supported.

Loading rulesets...
  Loading ruleset file: ./acls/valid.example.acl
  Ruleset file loaded successfully from yaml.
Ruleset(
  allow icmp outside any inside 8
  deny tcp outside any inside 22
  allowlog ip outside any inside 80
  allowlog ip outside any inside 443
  denylog udp outside any inside 161
  denylog udp outside any inside 162
  deny ip outside any inside any
)
  Loading ruleset file: ./acls/invalid.example.acl
* Ruleset issues found while parsing:
./acls/invalid.example.acl:1:7  ProtocolUnsupported: expected 'ip', 'tcp', 'udp', or 'icmp'
./acls/invalid.example.acl:2:0  ActionInvalid: expected 'allow', 'deny', 'allowlog', or 'denylog'
./acls/invalid.example.acl:3:21 PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
./acls/invalid.example.acl:4:32 PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
./acls/invalid.example.acl:5:23 RuleLengthErr: expected 6 fields
./acls/invalid.example.acl:6:30 PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
./acls/invalid.example.acl:7:18 PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'

Invalid rules provided in rulesets.
Unable to generate output with provided configuration and rulesets.
```

## Tests

```
$ cargo fmt && cargo test
[...]
running 19 tests
test device::tests::build_path_errs_on_invalid_iface ... ok
test ruleset::tests::action_parse_err ... ok
test ruleset::tests::dst_port_invalid ... ok
test ruleset::tests::portlist_expansion_invalid ... ok
test ruleset::tests::portlist_expansion_valid ... ok
test config::tests::device_has_valid_name ... ok
test ruleset::tests::portmap_list_invalid ... ok
test config::tests::device_has_invalid_name ... ok
test ruleset::tests::portmap_num_invalid ... ok
test ruleset::tests::portmap_list_valid ... ok
test ruleset::tests::portmap_range_invalid ... ok
test ruleset::tests::portmap_range_valid ... ok
test ruleset::tests::portmap_num_valid ... ok
test ruleset::tests::protocol_parse_err ... ok
test ruleset::tests::portmap_rangelist_valid ... ok
test ruleset::tests::rule_contains_multiple_lists ... ok
test ruleset::tests::src_port_invalid ... ok
test ruleset::tests::rule_lengths_invalid ... ok
test device::tests::build_device_succeeds ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Size

```
$ ls -lh target/release/am3k
-rwxrwxr-x 2 whoami whoami 3.1M Aug  8 08:32 target/release/am3k
```
