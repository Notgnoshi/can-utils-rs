# candumpr UX

## Status

**DRAFT**

## Scope

This document defines the user-facing features, CLI interface, and configuration file format for
candumpr, a CAN bus logging tool. It does not cover internal implementation details.

candumpr is an opinionated replacement for can-utils `candump`, focused on J1939 networks. It
prioritizes performance and multi-network support at the cost of broader CAN compatibility.

A primary design goal is lossless capture: candumpr should never drop a CAN frame under normal
operating conditions, including during log file rotation. Every frame that the kernel delivers to
the socket should appear in the output.

An additional convenience is to optionally send a J1939 address claim PGN request to ensure that the
CAN logs include address claims for every control function near the beginning of every log.

## Features

### Frame support

* Only supports CAN with 29-bit extended (J1939) identifiers.
* CAN FD and CAN XL are not supported.
* Error frames are supported and logged alongside data frames.

### Multi-interface logging

* Supports logging from an arbitrary number of CAN interfaces simultaneously.
* Each interface can be independently configured with its own filters and settings.
* Interfaces can be specified on the CLI, in a TOML config file, or both.

### Filtering

Two filtering mechanisms are supported. Both can be used together.

**candump-compatible mask filters** are specified per-interface using the same syntax as candump:

* `id:mask` -- positive match (accept when `received_id & mask == id & mask`)
* `id~mask` -- inverse match (accept when `received_id & mask != id & mask`)
* `#error_mask` -- error frame class filter (see `linux/can/error.h`)

All values are hexadecimal. Multiple filters are comma-separated after the interface name. Appending
`j` or `J` to the filter list switches that interface from OR to AND semantics (same as candump).

**Convenience filters** provide a more ergonomic way to filter J1939 traffic. These are specified in
the TOML config file:

* Filter by PGN (Parameter Group Number)
* Filter by source address
* Future work: filter by ISONAME + mask
* Toggle error frame capture on or off

Convenience filters are compiled to socket-level `id:mask` filters internally.

When no filters are specified, all traffic is accepted.

#### Filter combination semantics

When multiple filters are specified on the same interface (whether candump-style masks, convenience
filters, or both), they are combined with OR by default: a frame is accepted if it matches any
filter.

To switch to AND semantics (a frame must match all filters):

* On the CLI, append `j` to the candump-style filter list (e.g., `can0,...,j`)
* In the TOML config, set `filter_join = "and"` on the interface or in `[defaults]`

Both map to the `CAN_RAW_JOIN_FILTERS` socket option.

### Output formats

candumpr supports multiple output formats, configurable per-interface:

* **candump** (`.log`) -- default -- the can-utils `candump -L` log file format:
  `(1345212884.318850) can0 18FECA00#0011223344556677`
* **candump-tty** (`.log`) -- the can-utils `candump` console format:
  `can0  18FECA00   [8]  00 11 22 33 44 55 66 77`
* **ASC** (`.asc`) -- Vector ASCII logging format, compatible with CANalyzer/CANoe and other tools
  that import ASC files.
* **PCAP** (`.pcap`) -- packet capture format, compatible with Wireshark and tcpdump.

When compressed, an additional `.zst` suffix is appended (e.g., `.log.zst`, `.asc.zst`).

### Timestamps

Timestamp mode controls how frame timestamps are displayed in candump and candump-tty output
formats. ASC and PCAP use their native timestamp conventions and ignore this setting.

* **absolute** -- seconds since epoch with fractional seconds
* **delta** -- time elapsed since the previous received frame
* **zero** -- time elapsed since the first received frame

Hardware timestamps from the CAN controller are used automatically when available, falling back to
kernel software timestamps with a diagnostic warning. This requires no configuration.

### Clock correctness

candumpr is designed to start early in the boot cycle on IoT devices that may lack a persistent RTC.
On these devices, `CLOCK_REALTIME` can be invalid (near epoch) until NTP or another time source
synchronizes it.

candumpr will provide options to control how it detects an invalid clock and what it does with
frames captured before the clock becomes valid. Detection methods include a heuristic (is the clock
before a reasonable threshold?) and waiting for a clock step event. Behaviors may include dropping
frames, queueing them in memory, using zero-based timestamps, inserting a marker, or rotating the
log file when the clock becomes valid. The available behaviors may depend on the output format.

One strategy for clock correctness is to give each log file a strictly monotonic incrementing index.
Then at least you can tell the order of the files. candumpr should also attempt to detect and log
clock jumps to stderr so that they're less surprising if you have to reverse engineer what the clock
did by looking at strictly just the logs.

This feature requires dedicated detailed design and is not fully specified here.

### File logging and rotation

When logging to files, each monitored interface writes to its own log file. This applies even when
using the `any` interface binding; frames are separated by their source interface, and `{interface}`
resolves to the actual interface name (e.g., `can0`), not `any`.

* Log filenames are controlled by a format string with placeholders:
  * `{interface}` -- the source interface name (e.g., `can0`)
  * `{start-unix}` -- Unix seconds when the log file was opened (e.g., `1741868400`)
  * `{start-iso}` -- ISO 8601 timestamp when the log file was opened, without colons (e.g.,
    `2026-03-13T120000Z`), since colons break rsync and some filesystems.
  * Default format: `candumpr-{interface}-{start-unix}` (plus the appropriate file extension).
* The log directory path supports the same `{interface}` placeholder, allowing per-interface
  directory organization (e.g., `/var/log/candumpr/{interface}/`).
* If the resolved file path (directory + name + extension) would be identical for two or more
  interfaces, candumpr exits with a configuration error. Disambiguation can be achieved by including
  `{interface}` in the filename or directory path, or by setting different `log_dir` values
  per-interface.
* File rotation can be triggered by:
  * A time interval (e.g., `1h`, `30m`)
  * A file size threshold (e.g., `50MB`, `1GB`)
  * The value is unambiguous: size units (`B`, `KB`, `MB`, `GB`) and time units (`s`, `m`, `h`, `d`)
    do not overlap. Bare integers without a unit suffix are rejected.
  * SIGHUP is always available for manual rotation regardless of the configured method.
* During rotation, no frames are lost. Buffered frames are flushed to the old file before the new
  file begins.
* Completed log files are never partially written. Files are written to a temporary name and renamed
  atomically on completion.
* ZSTD streaming compression is optionally applied during writing.
* Buffered output is flushed to disk periodically (configurable interval) to limit data loss on
  unexpected power loss or crash.

When not logging to files, output goes to stdout.

### Log retention

When logging to files, candumpr can automatically prune old log files to prevent unbounded disk
usage.

* **max_total_size** -- maximum total size of all completed log files across all interfaces (e.g.,
  `10GB`). When exceeded, the oldest completed log files are deleted regardless of which interface
  produced them. Retention is checked after each log rotation.

### J1939 address claim

On startup and after each log rotation, candumpr can optionally broadcast a J1939 Address Claim PGN
request. This causes all devices on the bus to re-announce their addresses, ensuring each log file
contains a complete picture of which source addresses are in use.

### Statistics

Per-interface statistics counters are maintained and can be reported:

* Frame count (total and per-second)
* Byte count and estimated bitrate
* Dropped frame count (frames lost due to socket buffer overflow)

Dropped frame monitoring is always enabled.

### Socket configuration

* The socket receive buffer size can be configured per-interface. The tool attempts `SO_RCVBUFFORCE`
  first (requires `CAP_NET_ADMIN`) and falls back to `SO_RCVBUF`.

### Device resilience

* If a monitored CAN interface goes down, candumpr continues running and resumes logging when the
  interface comes back up. This is the default and only behavior (unlike candump, which exits by
  default).

### Signal handling

* **SIGHUP** -- trigger log file rotation
* **SIGTERM / SIGINT** -- graceful shutdown (flush buffers, finalize current log file)

### Diagnostic logging

Operational events are logged to stderr via `tracing`:

* Dropped frames (socket buffer overflow)
* Bus-off state changes and recovery
* Network interface up/down events
* Startup and shutdown status
* Log file rotation events

This keeps CAN data output (stdout or log files) clean, while ensuring operational issues are
visible. The log level can be set via `--log-level` on the CLI, `log_level` in the TOML config, or
the `CANDUMPR_LOG` environment variable (in `EnvFilter` format). The environment variable takes
precedence when set.

### Display options (stdout only)

When outputting to a TTY:

* Color mode (`--color`):
  * `never` -- no color or styling
  * `network` -- each interface gets a distinct color applied to the entire line, to visually
    distinguish traffic from different networks
  * `highlight` -- use color and weight to improve readability: the interface name and timestamp are
    colored, and data bytes alternate between bold and normal weight to make it easier to visually
    parse byte boundaries
* TX/RX direction is always shown for each frame.

## CLI interface

```
candumpr [OPTIONS] [INTERFACE[,FILTER]...]
```

### Positional arguments

Interfaces are specified as positional arguments, optionally followed by comma-separated
candump-compatible filters. The special name `any` receives from all CAN interfaces (same as
candump):

```sh
# Listen on all CAN interfaces that are up
candumpr any

# No filters (accept all traffic on both interfaces)
candumpr can0 can1

# candump-compatible mask filters
candumpr can0,18FECA00:1FFFFFFF can1,18FEE500:1FFFFFFF

# Inverse match
candumpr can0,18FECA00~1FFFFFFF

# Error frame filter
candumpr can0,#FFFFFFFF

# Join filters with AND semantics (must match all)
candumpr can0,18FECA00:1FFF0000,00000017:000000FF,j
```

### Options

#### Configuration

| Flag                  | Description                                  |
| --------------------- | -------------------------------------------- |
| `-C, --config <path>` | Path to a TOML configuration file            |
| `--log-level <level>` | Diagnostic log level (e.g., `info`, `debug`) |

CLI flags apply globally to every interface. Per-interface configuration, filtering, file logging
options (directory, naming, rotation, compression, retention), and socket tuning require a TOML
config file. Interfaces specified on the CLI are merged with interfaces in the config file.

#### Output format

| Flag                     | Description                                                        |
| ------------------------ | ------------------------------------------------------------------ |
| `-f, --format <fmt>`     | Output format: `candump`, `candump-tty`, `asc`, `pcap`             |
| `-t, --timestamp <type>` | Timestamp mode: `absolute`, `delta`, `zero` (candump formats only) |
| `-c, --color <mode>`     | Color mode: `never`, `network`, `highlight`                        |

#### File logging

| Flag | Description                                             |
| ---- | ------------------------------------------------------- |
| `-l` | Log to files in the current directory (default: stdout) |

#### J1939

| Flag                    | Description                                              |
| ----------------------- | -------------------------------------------------------- |
| `-A`, `--address-claim` | Send address claim request on startup and after rotation |

#### Termination

| Flag                 | Description                                              |
| -------------------- | -------------------------------------------------------- |
| `-n, --count <n>`    | Exit after receiving n frames                            |
| `-T, --timeout <ms>` | Exit if no frames received within this many milliseconds |

## TOML configuration file

The `[defaults]` section provides default values for all interface settings. Individual
`[interfaces.<name>]` sections can override any default. All fields are optional at every level.

```toml
log_level = "info" # diagnostic log level for stderr output
# All logs together must stay below this limit
max_total_size = "10GB"

[defaults]
# Output
format = "candump" # "candump" | "candump-tty" | "asc" | "pcap"
timestamp = "absolute" # "absolute" | "delta" | "zero" (candump formats only)
color = "highlight" # "never" | "network" | "highlight"

# File logging
log_dir = "/var/log/candumpr" # supports {interface} placeholder
log_name = "candumpr-{interface}-{start-unix}" # placeholders: {interface}, {start-unix}, {start-iso}
rotate = "1h" # time or size based rotation
compress = "none" # "zstd" | "none"
zstd_level = 3
flush_interval = "5s"

# Filtering
error_frames = true
pgns = []
source_addresses = []
filter_join = "or" # "and" | "or"

# Socket
recv_buffer = "2MB"

# J1939
address_claim = true

# --- Per-interface overrides ---

# Inherits all [defaults], overrides nothing:
[interfaces.can0]
# Overrides specific settings:
[interfaces.can1]
error_frames = false
pgns = [0xFECA, 0xFEE5]

[interfaces.can2]
address_claim = false
source_addresses = [0x00, 0x17]
log_dir = "/var/log/candumpr/can2"

# candump-compatible raw filters:
[interfaces.can3]
filters = ["18FECA00:1FFFFFFF", "18FEE500~1FFFF00"]

# AND semantics for all filters on this interface:
[interfaces.can4]
pgns = [0xFECA]
source_addresses = [0x17]
filter_join = "and"
```

### Precedence

Settings are resolved in this order, highest priority first:

1. CLI flags
2. TOML `[interfaces.<name>]`
3. TOML `[defaults]`
4. Built-in defaults

For settings available on the CLI, CLI flags apply globally and override all other sources,
including per-interface TOML settings. For example, `--format pcap` forces that format on every
interface. Most settings are only available through the TOML config file.

List-valued options (`pgns`, `source_addresses`, `filters`) are replaced wholesale at each
precedence level, not merged. For example, if `[defaults]` sets `pgns = [0xFECA, 0xFEE5]` and
`[interfaces.can0]` sets `pgns = [0xFECA]`, then `can0` uses only `[0xFECA]`.

### Interface discovery

Interfaces to monitor are the union of:

* Interfaces named on the CLI
* Interfaces listed in `[interfaces]` in the config file

The special name `any` is specified on the CLI only (`candumpr any`). It binds to all CAN
interfaces, including interfaces that come up after candumpr has started. Using `any` and named
interfaces together is a configuration error, since the `any` binding would duplicate frames from
explicitly-bound interfaces. When using `any`, settings come from `[defaults]` (and CLI flags).

Even when using `any`, log files are written per source interface (not a single combined file).

At least one interface must be specified.
