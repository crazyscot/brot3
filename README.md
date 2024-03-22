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

Use `cargo build` or `cargo build --locked` in the usual way.
There are unit tests, benchmarks and a reasonably strict `clippy` config.

There is no packaging configuration set up for the CLI at present.

## GUI

Prerequisites:
* All engine prerequisites (see above)
* Tauri prerequisites for your platform, if any (check Tauri documentation; on Linux some developer libraries are necessary)
* yarn ([classic](https://classic.yarnpkg.com/lang/en/docs/install/))

The following commands are run from the `ui` directory:

* To install the node packages: `yarn install --immutable`
* To run in development mode: `yarn tauri dev`. There is live reload.
  * To run without updating node packages: `yarn tauri dev -- -- --locked`
* To build and bundle the application: `yarn tauri build`. This takes a while because we enable LTO in this configuration.
  * To build without updating node packages: `yarn tauri build -- -- --locked`

In release mode, Tauri builds the GUI application for the target, plus one or more bundles.
* Linux: .deb package and appImage (standalone)
* OSX: .dmg file
* Windows: .msi installer package
