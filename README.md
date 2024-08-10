# am3k (WIP)

access control list manager 3000

## Brief

acl creation kit for managing rulesets at scale

## Usage

```
$ target/release/am3k -h
(am3k) Access Control List Manager 3000

Usage: am3k [OPTIONS] <FILE>

Arguments:
  <FILE>  Sets a custom config file

Options:
  -d, --debug    Print debug information
  -v, --verbose  Print verbose information
  -h, --help     Print help
  -V, --version  Print version

Environment:
    AM3K_PLATFORMS_PATH     Path to the directory containing platform definitions. Defaults to "./platform".
    AM3K_RULESETS_PATH      Path to the directory containing ACL definitions. Defaults to "./acls".
    AM3K_TEMPLATES_PATH     Path to the directory containing template definitions. Defaults to "./tmpl".
```

## Examples

### Valid

```
$ target/release/am3k site/example.yaml

Loading configuration file site/example.yaml...
Configuration file loaded successfully from yaml.

Checking platform is supported...
Platform is supported.

Loading rulesets...
Valid rules provided in rulesets.


rsk101-ext-fw1:
  type: juniper
  desc: srx1500
  interfaces: [ ae101, ae102, ae201, ae202, ]
  egress:
    interfaces: [ae201, ae202]
    ruleset: [valid.example]
    filters:
      dst: [example]
      src: [example]
  ingress:
    interfaces: [ae101, ae102]
    ruleset: [valid.example]
    filters:
      dst: [example]
      src: [example]

[### TRUNCATED ###]
```

### Invalid

```
$ target/release/am3k site/invalid.example.yaml -v

Loading configuration file site/invalid.example.yaml...
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
  Unable to find supported model [qfx5120] in [./platform]
ModelNotSupported: see `Device Onboarding` for more information
Platform is not supported.

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
test device::tests::build_device_succeeds ... ok
test ruleset::tests::action_parse_err ... ok
test ruleset::tests::dst_port_invalid ... ok
test ruleset::tests::portlist_expansion_invalid ... ok
test device::tests::build_path_errs_on_invalid_iface ... ok
test ruleset::tests::portlist_expansion_valid ... ok
test ruleset::tests::portmap_list_invalid ... ok
test ruleset::tests::portmap_list_valid ... ok
test ruleset::tests::portmap_num_invalid ... ok
test ruleset::tests::portmap_num_valid ... ok
test ruleset::tests::portmap_range_invalid ... ok
test ruleset::tests::portmap_range_valid ... ok
test ruleset::tests::portmap_rangelist_valid ... ok
test config::tests::device_has_valid_name ... ok
test ruleset::tests::protocol_parse_err ... ok
test config::tests::device_has_invalid_name ... ok
test ruleset::tests::rule_contains_multiple_lists ... ok
test ruleset::tests::rule_lengths_invalid ... ok
test ruleset::tests::src_port_invalid ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Size

```
$ ls -lh target/release/am3k
-rwxrwxr-x 2 whoami whoami 8.1M Aug 10 23:10 target/release/am3k
```
