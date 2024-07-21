# ACL Builder

## Examples

### Usage

``` zsh
$ target/release/acl-builder --help

Usage: acl-builder <config> [OPTIONS]

Options:
  -d, --debug       Enable debug mode for additional logging and diagnostic information.
  -h, --help        Print this help message and exit.

Arguments:
  <config>          Path to the yaml configuration file.

Examples:
  acl-builder config.yaml
  acl-builder config.yaml --debug
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

``` zsh
$ target/release/acl-builder site/example.valid.yaml

Loading configuration file site/example.valid.yaml...
Configuration file loaded without issue.

Checking configuration file components are valid:

1. Checking platform and model are supported...
Deployments for junos srx1500 are supported.

2. Checking devicelist against device naming convention...
Valid device names per naming convention

3. Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

4. Checking interfaces assignments for egress...
Valid interface assignments for egress.

5. Checking all generics are valid rules...
[
    "allow icmp outside any inside 8",
    "deny tcp outside any inside 22",
    "allowlog ip outside any inside 80,443",
    "denylog udp outside any inside 161-162",
    "deny ip outside any inside any",
]
Valid rules provided in generics.
```

### Invalid Config

``` zsh
$ target/release/acl-builder site/example.invalid.yaml

Loading configuration file site/example.invalid.yaml...
Configuration file loaded without issue.

Checking configuration file components are valid:

1. Checking platform and model are supported...
Deployments for junos srx1500 are supported.

2. Checking devicelist against device naming convention...
Valid device names per naming convention

3. Checking interfaces assignments for ingress...
Valid interface assignments for ingress.

4. Checking interfaces assignments for egress...
Valid interface assignments for egress.

5. Checking all generics are valid rules...
[
    "allow icmps outside any inside 8",
    "denys tcp outside any inside 22",
    "allowlog ip outside anys inside 80,443",
    "denylog udp outside any inside $",
    "test inside to outside",
]

ProtocolParseErr: expected 'ip', 'tcp', 'udp', or 'icmp' on
  - allow icmps outside any inside 8

ActionParseErr: expected 'allow', 'deny', 'allowlog', or 'denylog' on
  - denys tcp outside any inside 22

SrcPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any' on
  - allowlog ip outside anys inside 80,443

DstPortInvalid: expected a port (0-65535), range of ports, comma-separated list of ports, or 'any' on
  - denylog udp outside any inside $

RuleLengthErr: expected 6 fields, got 4 on
  - test inside to outside
thread 'main' panicked at src/main.rs:163:13:
 - GenericsRuleParser found 5 errors. Please update rules.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
