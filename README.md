An interactive fractal explorer.

This is my third Mandelbrotter.
[brot2](https://github.com/crazyscot/brot2) was the previous incarnation.

![A close-up of the original Mandelbrot set. Origin=-1.259742+0.377104i, axes=0.01+0.01i](brot3.jpg)

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

## Releasing

Prerequisites:
* `cargo install git-cliff` and ensure that `~/.cargo/bin` is on your PATH.

Steps:
* Create release changeset
  * Update project version in `cargo.toml`
  * `git cliff --tag v<intended new tag> > CHANGELOG.md`
  * commit with prefix `chore(release)`
* Merge changeset to main as usual
* Update `release` branch to the desired release point, push it to github.
* ✨✨ automation happens ✨✨
* Edit and publish the draft release in Github. You may find it useful to press the button to autogenerate the release notes; `git cliff` might also yield insights.
