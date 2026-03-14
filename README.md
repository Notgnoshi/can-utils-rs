# can-utils-rs

Opinionated rewrites of can-utils in Rust

## Purpose

[can-utils](https://github.com/linux-can/can-utils) is great. I use it _a lot_. This is not a clone
of can-utils. This is an opinionated rewrite with different capabilities and different constraints.

## Target environment

A modern-ish Linux with io_uring and socketcan available. A ~4 core ~1GHz arm64 CPU with 1GB memory
and 4+ J1939 CAN networks.
