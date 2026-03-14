# candumpr clock correctness

## Status

**TODO**

## Scope

This document specifies how candumpr handles unreliable system clocks, covering:

* Detection of an invalid `CLOCK_REALTIME` (heuristic threshold, clock step events)
* Behavior for frames captured before the clock becomes valid
* Monotonic file indexing to preserve ordering independent of wall clock
* Clock jump detection and diagnostic logging
* Interaction with log file rotation and timestamps
* Interaction with output formats (candump, ASC, PCAP) that embed timestamps
