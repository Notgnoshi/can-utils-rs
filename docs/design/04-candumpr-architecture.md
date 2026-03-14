# candumpr architecture

## Status

**TODO**

## Scope

This document specifies the internal architecture of candumpr, covering the threading model, I/O
strategy, and the mechanisms used to achieve lossless capture. It does not cover user-facing
features or CLI/config design (see [01-candumpr-ux](01-candumpr-ux.md)).

## Target environment

A modern-ish Linux with io_uring and socketcan available. A ~4 core ~1 GHz arm64 CPU with 1 GB
memory and 4+ J1939 CAN networks.

## Design goal: never drop a frame

TODO: Define what "never drop" means precisely. Kernel socket buffer overflow is the primary
mechanism for frame loss. Describe the end-to-end path from kernel socket buffer to flushed bytes on
disk, and identify every point where frames could be lost or delayed.

## Option 1: dedicated thread pairs

One recv thread and one write thread per interface. The recv thread reads frames from the socket and
passes them to its paired write thread over a channel. The write thread handles formatting,
compression, and file I/O.

TODO: Describe how io_uring fits in (recv side, write side, or both). Describe the channel type and
backpressure strategy. Describe how log rotation and SIGHUP are coordinated between the two threads.

## Option 2: shared threads

A small number of shared recv threads and shared write threads, rather than a dedicated pair per
interface. This may be a better fit for the target environment of 4 ARM cores with 4+ interfaces,
where dedicating 2 threads per interface would oversubscribe the CPU.

TODO: Describe the multiplexing strategy (io_uring multishot recv, epoll, etc.). Describe how write
work is distributed. Describe how this interacts with per-interface file handles, rotation, and
compression state.

## Back-of-the-napkin math

TODO: Estimate the frame rate per interface (J1939 250 kbit/s, 8 byte payloads, 29-bit IDs).
Estimate the CPU cost per frame for recv, formatting, and write. Estimate the throughput ceiling for
each option on the target hardware. Identify whether the bottleneck is CPU, memory bandwidth, or
I/O.

## Benchmarking strategy

TODO: Define how to benchmark the two options against each other. Describe the test setup (vcan,
cangen, real hardware). Define the metrics to collect (frame loss, latency, CPU usage, memory
usage). Define the workload (number of interfaces, frame rate, payload size). Define the acceptance
criteria.

## Open questions

TODO
