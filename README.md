# am3k

acl manager 3000

## Brief

a work in progress acl creation kit

a project that mainly exists for me to learn more rust

## Examples

### Usage

```
$ target/release/acl-builder -h

Usage: acl-builder <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for additional logging and diagnostic information.
  -h, --help        Print this help message and exit.

Arguments:
  <config>          Path to the yaml configuration file.

Examples:
  acl-builder config.yaml
  acl-builder config.yaml -d

Description:
  The acl-builder tool is used to build and manage access control lists (ACLs) based on the provided configuration file.
  The configuration file should be a YAML file specifying the rules and settings for the ACL.

  - The <config> argument is mandatory and specifies the path to the configuration file.
  - Use the -d or --debug option to enable debug mode, which provides additional output useful for debugging.

Notes:
  - Ensure the configuration file is correctly formatted as a YAML file.
  - The tool will output the resulting ACL to the standard output or to a specified file as configured.

For more information, visit: [[ NotYetImplementedError ]]
```

### Valid Config

```
$ target/release/acl-builder site/example/valid.yaml

Loading configuration file site/example/valid.yaml...
Configuration file loaded successfully from yaml.

Checking devicelist naming convention...
Valid device names per naming convention.

Checking device is supported...
Platform and model are supported.

Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

Checking interfaces assignments for egress...
Valid interface assignments for egress.

Checking all rules are valid...
Valid rules provided in rules.
```

### Invalid Rules

```
$ target/release/acl-builder site/example/rules.invalid.yaml

Loading configuration file site/example/rules.invalid.yaml...
Configuration file loaded successfully from yaml.

Checking devicelist naming convention...
Valid device names per naming convention.

Checking device is supported...
Platform and model are supported.

Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

Checking interfaces assignments for egress...
Valid interface assignments for egress.

Checking all rules are valid...
site/example/rules.invalid.yaml:2 :: ProtocolUnsupported: expected 'ip', 'tcp', 'udp', or 'icmp'
site/example/rules.invalid.yaml:3 :: ActionInvalid: expected 'allow', 'deny', 'allowlog', or 'denylog'
site/example/rules.invalid.yaml:4 :: PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:5 :: PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:6 :: RuleLengthErr: expected 6 fields
site/example/rules.invalid.yaml:7 :: PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:8 :: PortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
[ERROR]: 7 configuration issues found while parsing rules.
```

## Tests

```
$ cargo fmt && cargo test
[...]
running 22 tests
test device::tests::device_has_invalid_iface ... ok
test config::tests::device_has_valid_name ... ok
test config::tests::device_has_invalid_name ... ok
test device::tests::device_has_valid_iface ... ok
test ruleset::tests::action_parse_err ... ok
test ruleset::tests::dst_port_invalid ... ok
test ruleset::tests::portlist_expansion_invalid ... ok
test ruleset::tests::portlist_expansion_valid ... ok
test ruleset::tests::portmap_list_invalid ... ok
test ruleset::tests::portmap_list_valid ... ok
test ruleset::tests::portmap_num_invalid ... ok
test ruleset::tests::portmap_num_valid ... ok
test ruleset::tests::portmap_range_invalid ... ok
test ruleset::tests::portmap_range_valid ... ok
test ruleset::tests::portmap_rangelist_valid ... ok
test ruleset::tests::protocol_parse_err ... ok
test ruleset::tests::rule_contains_multiple_lists ... ok
test ruleset::tests::rule_lengths_invalid ... ok
test ruleset::tests::src_port_invalid ... ok
test junos::tests::ptx1000_valid_regex ... ok
test junos::tests::qfx5200_valid_regex ... ok
test junos::tests::srx1500_valid_regex ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Size

```
$ ls -lh target/release/acl-builder -h
-rwxrwxr-x 2 whoami whoami 3.0M Jul 30 08:00 target/release/acl-builder
```
