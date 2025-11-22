# brot3

An interactive fractal explorer.

This is my third Mandelbrotter.
[`brot2`](https://github.com/crazyscot/brot2) was the previous incarnation, an interactive GTK+ application.
The original `brot` was written in C, output only to the terminal, and has long since been lost to the mists of time.

![A screenshot of the brot3 window showing a zoom into the Mandelbrot set.](brot3c.jpg)

This time around it's in Rust and has quite the history.

- brot3 1.0 started out using Tauri and OpenSeadragon for its UI.
- Version 2.0 (never released) was an attempt to rework the GUI using slint.
- The 3.0 branch is a significant overhaul, rendering on the GPU using [rust-gpu](https://github.com/rust-gpu/rust-gpu), with GUI using [egui](https://crates.io/crates/egui).

More notes to come here when things are in less of a state of flux...

## Supported systems

Primary:

- Debian (I develop on 13/trixie; releases are built on Ubuntu 22.04)

Secondary:

- OSX (minimum Rust supported versions apply; currently 10.12 on x86_64 and 11.0 on aarch64)
- Windows 11

Binary releases can be found in [github releases](https://github.com/crazyscot/brot3/releases/).

## Experimenting

### Prerequisites

You only really need a recent version of the `cargo` tool.

We use a [toolchain file](rust-toolchain.toml) to select the rust-gpu recommended toolchain, which is currently a specific nightly build that you're unlikely to have to hand. This is required as rust-gpu makes significant use of compiler internals.

### Building

**Cargo will automatically install the required nightly toolchain if you don't already have it.**

`cargo run --locked` will launch the GUI in interactive mode. By default this runs with the `hot-reload-shader` feature.
As you might imagine, this rebuilds and reloads the shader on save, and adds the cost of the spirv-builder to compile-time and binary size.

The useful feature flag combinations are:

| Flags passed to cargo                          | Result                                                                                             |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `--no-default-features`                        | Shader compiled at build time only: **Recommended if you only want to browse the Mandelbrot set!** |
| _None_                                         | Runtime shader compilation with hot reload                                                         |
| `--no-default-features -F runtime-compilation` | Runtime shader compilation without hot reload                                                      |

To build a standalone application binary, `cargo build --locked --no-default-features` is usually what you wanted as it disables runtime shader compilation.

There are limited unit tests and benchmarks. More may be added later.

### Speeding up build times

You may care to use an alternative linker to improve build times.

For example, if you want to use the [wild linker](https://github.com/davidlattimore/wild) (`cargo binstall wild-linker`) this is what you might put in your `~/.cargo/config.toml`:

```
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=--ld-path=wild"]
```

## Maintainer notes

See [MAINTENANCE.md](MAINTENANCE.md).
