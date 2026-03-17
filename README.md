# candemonium

![lint workflow](https://github.com/Notgnoshi/candemonium/actions/workflows/lint.yml/badge.svg?event=push)
![release workflow](https://github.com/Notgnoshi/candemonium/actions/workflows/release.yml/badge.svg?event=push)
![code coverage](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/Notgnoshi/55f3f6cae2abdc5d011d907624dfb883/raw/can-utils-rs-coverage.json)

Opinionated CAN utils written in Rust.

## Purpose

[can-utils](https://github.com/linux-can/can-utils) is great. I use it _a lot_. This is not a clone
of can-utils. This is an opinionated set of tools with different capabilities and different
constraints.

## Target environment

A modern-ish Linux with io_uring and socketcan available. A ~4 core ~1GHz arm64 CPU with 1GB memory
and 4+ J1939 CAN networks.
