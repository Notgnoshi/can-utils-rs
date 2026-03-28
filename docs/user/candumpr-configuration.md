# candumpr configuration

**TODO**

This document describes the TOML configuration file format for candumpr, covering:

* File structure: top-level settings, `[defaults]`, and `[interfaces.<name>]` sections
* All available configuration keys and their types, defaults, and valid values
* Precedence rules: CLI flags, per-interface overrides, defaults, built-in defaults
* List-valued options: replace (not merge) semantics
* Interface discovery: CLI and TOML union, `any` constraints
* Validation rules and error conditions
* Example configurations for common deployment scenarios
