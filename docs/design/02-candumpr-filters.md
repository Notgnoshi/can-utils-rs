# candumpr filter syntax and semantics

## Status

**TODO**

## Scope

This document specifies the filter syntax and semantics for candumpr, covering:

* candump-compatible `id:mask` and `id~mask` filter syntax
* Error frame class filters (`#error_mask`)
* Convenience filters (PGN, source address)
* How convenience filters compile to kernel-level `CAN_RAW_FILTER` entries
* Filter combination semantics (OR vs AND, `CAN_RAW_JOIN_FILTERS`)
* Interaction between candump-style and convenience filters on the same interface
