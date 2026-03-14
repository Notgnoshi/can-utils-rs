# candumpr

## Features

* Only supports 29-bit J1939 standard IDs
* Configurable via a TOML file
* Support logging multiple interfaces
* Long-running daemon with file rotation on a timer, as well as logging to stdout in a TTY
* Optionally send a J1939 address claim PGN request on startup and after rotation to ensure address
  claims are present in each log
* Buffered and async writes to prevent dropping frames due to blocking
* Support multiple output formats, including can-utils log formats and PCAP.
* Statistics counters per device, including bitrate estimation
