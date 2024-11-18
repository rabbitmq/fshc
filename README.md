# fshc: a File and Socket Handle Counter

A tiny command line tool that prints file and socket descriptor (handle)
counts for a process.

Think of it as a very small, cross-platform, more machine-friendly version of `handle.exe`
that only does one thing (and hopefully does it well).

## Supported Operating Systems

 * Windows
 * Linux
 * macOS


## Binary Builds

See [Releases](https://github.com/rabbitmq/fshc/releases).


## Usage

``` shell
# formats the output using 'jq'
fshc --pid 73847 | jq
```

``` shell
# formats the output using 'jq'
fshc --pid 73847 --only-total | jq
```

## License

This tool is dual-licensed under the ASL2 and MIT licenses.

SPDX-License-Identifier: MIT OR Apache-2.0
