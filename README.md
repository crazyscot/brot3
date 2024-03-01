An interactive fractal explorer.

This is my third Mandelbrotter.
[brot2](https://github.com/crazyscot/brot2) was the previous incarnation.

This time it's in Rust, building a graphical UI using Tauri and OpenSeadragon.

More notes to come here when things are in less of a state of flux...

# Building

## Engine & CLI

Prerequisites:
* Rust toolchain (we target _stable_)
* (Linux only) `mold` (linker; see `.cargo/config.toml`)

Use `cargo build` in the usual way.
There are unit tests, benchmarks and a reasonably strict `clippy` config.

There is no packaging configuration set up for the CLI at present.

## GUI

Prerequisites:
* Rust toolchain
* Tauri prerequisites for your platform, if any (check Tauri documentation; on Linux some developer libraries are necessary)

All these commands are in the `ui` directory.

* To install the node packages: `yarn install --immutable`
* To run in development mode: `yarn tauri dev`. There is live reload.
* To build and bundle the application: `yarn tauri build`. This takes a while because we enable LTO in this configuration.

In release mode, Tauri builds the GUI application for the target, plus one or more bundles.
* Linux: .deb package and appImage (standalone)
* OSX: .dmg file
* Windows: .msi installer package
