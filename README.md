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
  AM3K_PLATFORMS_PATH     Path to the directory containing platform definitions. Defaults to "./platform/".

Examples:
  am3k config.yaml
  am3k config.yaml -d

Description:
  ACL Manager 3000 (am3k) is used to build and manage access control lists via provided configuration file.
  The configuration file should be a YAML file specifying the rules and settings for the ACLs.

For more information, visit: [NotYetImpld]

```

### Valid Config

```
$ target/release/am3k site/example/valid.yaml

Loading configuration file site/example/valid.yaml...
Configuration file loaded successfully from yaml.

Checking platform is supported...
Platform is supported.

Checking all rules are valid...
Valid rules provided in rules.

Expanding ruleset...
Ruleset(
  allow icmp outside any inside 8
  deny tcp outside any inside 22
  allowlog ip outside any inside 80
  allowlog ip outside any inside 443
  denylog udp outside any inside 161
  denylog udp outside any inside 162
  deny ip outside any inside any
)
Ruleset expanded.
```

### Invalid Rules

```
$ target/release/am3k site/example/rules.invalid.yaml
Verbose mode is enabled.

Loading configuration file site/example/rules.invalid.yaml...
  Checking devicelist naming convention...
  Valid device names per naming convention.
Configuration file loaded successfully from yaml.

Checking platform is supported...
  Loading path to supported platforms...
  Found path: ./platform/

  Searching for matching supported platform file...
  Found ./platform/juniper.yaml

  Loading supported platforms file...
  Platforms file loaded successfully from yaml.

  Checking supported model...
  Model supported.

  Confirming interfaces are valid...
  Interfaces are valid
Platform is supported.

Checking all rules are valid...
site/example/rules.invalid.yaml:2:11    ProtocolUnsupported: expected 'ip', 'tcp', 'udp', or 'icmp'
site/example/rules.invalid.yaml:3:5     ActionInvalid: expected 'allow', 'deny', 'allowlog', or 'denylog'
site/example/rules.invalid.yaml:4:25    PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:5:36    PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:6:28    RuleLengthErr: expected 6 fields
site/example/rules.invalid.yaml:7:34    PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:8:22    PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
Rule configuration issues found while parsing: 7
```

## Tests

```
$ cargo fmt && cargo test
[...]
running 19 tests
test config::tests::device_has_valid_name ... ok
test device::tests::build_path_errs_on_invalid_iface ... ok
test ruleset::tests::dst_port_invalid ... ok
test ruleset::tests::portlist_expansion_invalid ... ok
test ruleset::tests::portlist_expansion_valid ... ok
test ruleset::tests::portmap_list_invalid ... ok
test ruleset::tests::portmap_list_valid ... ok
test config::tests::device_has_invalid_name ... ok
test ruleset::tests::portmap_num_invalid ... ok
test ruleset::tests::portmap_num_valid ... ok
test ruleset::tests::portmap_range_invalid ... ok
test ruleset::tests::portmap_range_valid ... ok
test ruleset::tests::portmap_rangelist_valid ... ok
test ruleset::tests::protocol_parse_err ... ok
test ruleset::tests::rule_contains_multiple_lists ... ok
test ruleset::tests::rule_lengths_invalid ... ok
test ruleset::tests::src_port_invalid ... ok
test ruleset::tests::action_parse_err ... ok
test device::tests::build_device_succeeds ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Size

```
$ ls -lh target/release/am3k
-rwxrwxr-x 2 whoami whoami 3.1M Aug  4 01:45 target/release/am3k
```
