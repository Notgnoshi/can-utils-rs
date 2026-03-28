# Testing strategy

## Status

**IMPLEMENTED**

## Scope

This document describes the mechanisms available for writing tests around the utilities in this
project that depend on Linux socketcan interfaces that require either real hardware or elevated
permissions to create.

## Problem

These utilities interact directly with CAN sockets. Testing requires CAN interfaces, but:

* Real CAN hardware is not available in CI.
* Virtual CAN (vcan) interfaces require `CAP_NET_ADMIN` to create.
* vcan interfaces are system-global resources, so parallel tests using shared interfaces cause
  interference.
* Tests must run in CI (GitHub Actions) and locally without requiring root.

## Solution: user + network namespaces

Each test process enters its own namespace using `unshare(CLONE_NEWUSER | CLONE_NEWNET)`. Inside the
namespace, the process has `CAP_NET_ADMIN` without real root privileges, vcan interfaces are private
and isolated, and everything is cleaned up when the process exits. See the
[vcan-fixture](/vcan-fixture/src/lib.rs) crate for the implementation.

Depending on your system (Fedora 42 doesn't need the following, but Ubuntu 24.04 does), you may need
to disable the following apparmor setting:

```sh
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
```

Constraint: `unshare(CLONE_NEWUSER)` requires a single-threaded process. The Rust test harness is
multi-threaded, so namespace entry needs to happen in a `ctor` constructor before `main()`.

```rust
#[ctor::ctor]
fn setup() {
    tracing_subscriber::fmt()
        .with_test_writer()
        .init();
    vcan_fixture::enter_namespace();
}
```

## CI

Tests that require vcan use `#[cfg_attr(feature = "ci", ignore = "requires vcan")]`. In CI,
`--all-features` enables the `ci` feature, making them `#[ignore]`. They are then run as a separate
step gated on whether vcan setup succeeded.

A separate canary job (`vcan-available`) with `continue-on-error: true` fails with a warning status
when the vcan module is unavailable on the runner, rather than silently skipping the tests. This
makes it visible in the PR workflow status when vcan isn't available, but doesn't prevent merging
for infrastructure reasons outside of my control (I've ready about `linux-modules-extra` not always
matching the runner kernel version).

See [lint.yml](/.github/workflows/lint.yml) for the implementation.

## Benchmarking

There are additional utilities in the `vcan_fixture::bench` module for

* Querying current thread and process resource usage
* Pinning the current process to N CPU cores
* Starting a PWM-like busyloop thread to approximate P% CPU usage over N threads
