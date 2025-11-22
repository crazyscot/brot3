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

**N.B. You don't necessarily need a GPU card: I find this runs just fine on a PC with integrated graphics.** If rust-gpu can talk to it, it will probably work.

## Acknowledgements

Credit is due to [@abel465](https://github.com/abel465) for creating:

- [easy-shader-runner](https://github.com/abel465/easy-shader-runner), which provided a very useful way for me to get started with Rust on the GPU;
- His own [mandelbrot](https://github.com/abel465/mandelbrot) implementation, which I drew a lot of inspiration from;
- The [abels-complex](https://crates.io/crates/abels-complex) crate, which is GPU-friendly.

Credit is also due to the [rust-gpu](https://github.com/Rust-GPU/rust-gpu) team past and present for creating the platform. While Rust on the GPU does have a bit of a "cursed subset of the language" feel, which is inherent to the nature of GPU programming, I have found it workable and performant so far in this use case.

## Supported systems

Primary:

- Debian (I develop on 13/trixie; releases are built on Ubuntu 22.04)

Secondary:

- OSX (requires OSX 10.12 on x86_64, or 11.0 on Apple silicon)
- Windows 11

Binary releases can be found in [github releases](https://github.com/crazyscot/brot3/releases/).

### Hardware requirements

You need a GPU, or integrated graphics chip, that rust-gpu can talk to.

I have had success with the following:

- Intel iGPU (Ultra7 265K processor)
- AMD Radeon integrated graphics (Lenovo V15 laptop with Ryzen 7 CPU)
- nVidia RTX 3070 GPU (with AMD Ryzen 7 CPU)
- nVidia GT 755M (2014 iMac)

## Experimenting

### Prerequisites

You only really need a recent version of the `cargo` tool.

We use a [toolchain file](rust-toolchain.toml) to select the rust-gpu recommended toolchain, which is currently a specific nightly build that you're unlikely to have to hand. This is required as rust-gpu makes significant use of compiler internals.

### Building

**Cargo will automatically install the required nightly toolchain if you don't already have it.**

`cargo run --locked` will launch the GUI in interactive mode.

The useful feature flag combinations are:

| Flags passed to cargo    | Result                                                                                             |
| ------------------------ | -------------------------------------------------------------------------------------------------- |
| _None_                   | Shader compiled at build time only. **Recommended if you only want to browse the Mandelbrot set!** |
| `-F hot-reload-shader`   | Runtime shader compilation with hot reload                                                         |
| `-F runtime-compilation` | Runtime shader compilation without hot reload                                                      |

As you might imagine, the runtime compilation options add the cost of the spirv-builder to compile-time and binary size.
This is only useful if you want to hack on the shader.

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
