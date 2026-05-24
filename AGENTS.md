# AGENTS.md

## Project Overview

Fractal explorer and toolkit for the Mandelbrot set and related algorithms, offering:
- Interactive GUI mode
- Command-line rendering. Fractal parameters may be specified on the command line, or using a save file.

This software is written in Rust. Part of it runs on the GPU, using rust-gpu.

Crate structure:
- lib (`brot3-lib`): Fractal implementations. This crate includes all the GPU-resident code and shared structures
- ui (`brot3-ui`): GUI and command-line interface only
- easy-shader-runner: Framework for running shader
- shader_builder: Builds the shader
- xtask: Miscellaneous build helpers

Save file formats:
- JSON: serialized form of the `UiState` struct
- PNG with an embedded text chunk named `uistate`

## Code style

- Rust source files must be formatted using `rustfmt` or `cargo fmt`.
- TOML, YAML and various markup format files must be formatted using `dprint fmt`.
- Typo checks are performed by `typos`.

These are enforced by the pre-commit hook, which is `scripts/pre-commit`.

## Code standards

Code in `brot3-lib` is generally expected to have unit tests.
All unit tests must pass (`cargo nextest run`).

Coverage tracking is enabled but not enforced. `scripts/coverage` runs coverage analysis.

CI performs a more thorough set of checks including `cargo shear`, `cargo machete` and `cargo deny`.
