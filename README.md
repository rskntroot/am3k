# ACL Builder

## Examples

### Usage

``` zsh
$ target/release/acl-builder
thread 'main' panicked at src/main.rs:16:9:
Usage: target/release/acl-builder <input_yaml>
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### Invalid Config

``` zsh
$ target/release/acl-builder site/example.invalid.yaml

Checking all generics are valid rules...

ProtocolParseErr: expected 'allow', 'deny', 'allowlog', or 'denylog' on
  - allow icmps outside any inside 8

ActionParseErr: expected 'allow', 'deny', 'allowlog', or 'denylog' on
  - denys tcp outside any inside 22

SrcPortInvalid :: expected a port, range list, or 'any' on
  - allowlog ip outside anys inside 80,443

DstPortInvalid :: expected a port, range list, or 'any' on
  - denylog udp outside any inside $

RuleLengthErr :: Expected 5 fields, got 4 on
  - test inside to outside
thread 'main' panicked at src/main.rs:36:13:
GenericsRuleParser found 5 errors. Please update rules.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### Valid Config

``` zsh
$ target/release/acl-builder site/example.valid.yaml

Checking all generics are valid rules...
[
    Rule {
        action: Allow,
        protocol: ICMP,
        src_prefix: "outside",
        src_port: Any,
        dst_prefix: "inside",
        dst_port: Num(
            8,
        ),
    },
    Rule {
        action: Deny,
        protocol: TCP,
        src_prefix: "outside",
        src_port: Any,
        dst_prefix: "inside",
        dst_port: Num(
            22,
        ),
    },
    Rule {
        action: AllowLog,
        protocol: IP,
        src_prefix: "outside",
        src_port: Any,
        dst_prefix: "inside",
        dst_port: List(
            "80,443",
        ),
    },
    Rule {
        action: DenyLog,
        protocol: UDP,
        src_prefix: "outside",
        src_port: Any,
        dst_prefix: "inside",
        dst_port: Range(
            "161-162",
        ),
    },
    Rule {
        action: Deny,
        protocol: IP,
        src_prefix: "outside",
        src_port: Any,
        dst_prefix: "inside",
        dst_port: Any,
    },
]
Valid rules provided in generics.

Checking interfaces assignments for ingress...
 - 'ae10' matched '^(ae|lo)\d{1,3}(\.\d{1,3})?$'
Valid interface assignments for ingress.

Checking interfaces assignments for egress...
 - 'ae20' matched '^(ae|lo)\d{1,3}(\.\d{1,3})?$'
Valid port assignments for egress.
```