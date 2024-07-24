# ACL Builder

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
Configuration file loaded without issue.

Checking configuration file components are valid:

1. Checking devicelist against device naming convention...
Valid device names per naming convention

2. Checking platform and model are supported...
Platform and model are supported.

3. Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

4. Checking interfaces assignments for egress...
Valid interface assignments for egress.

5. Checking all rules are valid...
Valid rules provided in rules.
```

### Invalid Rules Config

```
$ target/release/acl-builder site/example/rules.invalid.yaml

Loading configuration file site/example/rules.invalid.yaml...
Configuration file loaded without issue.

Checking configuration file components are valid:

1. Checking devicelist against device naming convention...
Valid device names per naming convention

2. Checking platform and model are supported...
Platform and model are supported.

3. Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

4. Checking interfaces assignments for egress...
Valid interface assignments for egress.

5. Checking all rules are valid...
site/example/rules.invalid.yaml:2 :: ProtocolParseErr: expected 'ip', 'tcp', 'udp', or 'icmp'
site/example/rules.invalid.yaml:3 :: ActionParseErr: expected 'allow', 'deny', 'allowlog', or 'denylog'
site/example/rules.invalid.yaml:4 :: SrcPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:5 :: DstPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/rules.invalid.yaml:6 :: RuleLengthErr: expected 6 fields, got 4
thread 'main' panicked at src/main.rs:157:13:
 - RulesParser found 5 errors. Please update rules.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### Invalid Ports Config

```
$ target/release/acl-builder site/example/ports.invalid.yaml

Loading configuration file site/example/ports.invalid.yaml...
Configuration file loaded without issue.

Checking configuration file components are valid:

1. Checking devicelist against device naming convention...
Valid device names per naming convention

2. Checking platform and model are supported...
Platform and model are supported.

3. Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

4. Checking interfaces assignments for egress...
Valid interface assignments for egress.

5. Checking all rules are valid...
site/example/ports.invalid.yaml:2 :: ExpandingDstPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
site/example/ports.invalid.yaml:3 :: ExpandingSrcPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any'
thread 'main' panicked at src/main.rs:157:13:
 - RulesParser found 2 errors. Please update rules.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Tests

```
$ cargo fmt && cargo test
   Compiling acl-builder v0.1.0 (/home/lost/workspace/acl_builder)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.01s
     Running unittests src/main.rs (target/debug/deps/acl_builder-3493a91829d5c0f4)

running 16 tests
test device::tests::device_has_invalid_iface ... ok
test device::tests::device_has_valid_iface ... ok
test ruleset::tests::action_parse_err ... ok
test ruleset::tests::dst_port_invalid ... ok
test ruleset::tests::portlist_expansion_invalid ... ok
test ruleset::tests::portlist_expansion_valid ... ok
test junos::tests::ptx1000_valid_regex ... ok
test ruleset::tests::rule_contains_multiple_lists ... ok
test ruleset::tests::rule_length_err_long ... ok
test ruleset::tests::rule_length_err_short ... ok
test ruleset::tests::src_port_invalid ... ok
test ruleset::tests::protocol_parse_err ... ok
test junos::tests::srx1500_valid_regex ... ok
test junos::tests::qfx5200_valid_regex ... ok
test tests::device_has_invalid_name ... ok
test tests::device_has_valid_name ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
