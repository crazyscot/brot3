# Copilot Instructions

## Build, test, and lint commands

- The workspace is pinned to `nightly-2026-04-11` via `rust-toolchain.toml` because `brot3-lib` is also compiled to SPIR-V with rust-gpu. If builds fail with toolchain errors, ensure `rust-toolchain.toml` is present at the workspace root. If the file is missing, restore it from version control instead of creating it manually with a different toolchain version. It is not necessary to explicitly install the pinned toolchain as cargo will do that automatically.
- Run the app locally only with explicit non-GUI arguments: `cargo run --locked -p brot3-ui --bin brot3 -- <arguments>`. Do not run the app without arguments, as that opens GUI mode. This restriction applies to any direct invocation of the brot3 binary, including in scripts and CI steps
  - For headless render mode, pass at minimum `--input <state_file> --output <output.png>`. Example: `cargo run --locked -p brot3-ui --bin brot3 -- --input state.json --output out.png`.
  - Example state files can be found in the `data/` directory.
- Build the release binary with `cargo build --locked --release -p brot3-ui --bin brot3`.
- Build just the shader builder path used in CI with `cargo build --locked --release -p shader_builder --no-default-features -F use-installed-tools`.
- `cargo xtask ...` is available via the alias in `.cargo/config.toml`. Do not concern yourself with these unless explicitly asked to.

### Running tests

#### Prerequisites check

- Run `cargo nextest --version`

#### Running with nextest (only if prerequisites check passes)

- Run the engine/unit-test suite with `cargo nextest run -p brot3-lib`.
- Run a single lib test with `cargo nextest run -p brot3-lib 'test(~round_trip_basic)'` (replace `round_trip_basic` with a nextest filter expression such as `test(~substring)` or `test(/regex/)`; see `cargo nextest run --help` for filter syntax).

#### Running with cargo test (ignore unless prerequisites check fails)

- Run the engine/unit-test suite with `cargo test -p brot3-lib`.
- Run a single lib test with `cargo test -p brot3-lib round_trip_basic`.

#### Test coverage

Coverage is tracked for `brot3-lib`.
If `cargo llvm-cov` is not installed and the user explicitly asks you to install missing tools, install it with `cargo install cargo-llvm-cov` first.

Use the `scripts/coverage` script when running inside the repo locally.

Use `cargo llvm-cov -p brot3-lib --all-features --doctests --lcov --output-path lcov.info --locked` only when the script is unavailable or when you need a specific output path.

#### Tests in the UI crate

The only explicit UI-side test is an ignored compute-shader performance test: `cargo test -p brot3-ui --test compute_shader -- --ignored`.

### Linter checks

`scripts/pre-commit` is the local hook and runs `dprint check`, `typos`, and `cargo fmt --check` on staged files. If `dprint` or `typos` are not installed and the user explicitly asks you to install missing tools, install them with `cargo install dprint` and `cargo install typos-cli` before running the hook or the CI lint checks.

Run all CI lint commands in the order listed below. All are blocking failures. Stop at the first non-zero exit code, report the failing command and its output, and do not run subsequent commands. A command is a blocking failure only when it is installed and exits non-zero. Missing-tool exits are handled separately per the missing-tool instructions above.

If `cargo-machete`, `cargo-shear` or `cargo-deny` are not installed and the user explicitly asks you to install missing tools, install them with `cargo install dprint` and `cargo install typos-cli` before running the hook or the CI lint checks.

#### Formatting and style
- `cargo fmt --all -- --check`
- `dprint check --list-different`
- `typos`

#### Clippy
- `cargo clippy --locked --no-deps -p brot3-lib -p brot3-ui --no-default-features`
- `cargo clippy --locked --no-deps -p brot3-lib -p brot3-ui -F ui -F hot-reload-shader`

#### Unused dependencies

- `cargo +stable machete --with-metadata`
- `cargo +stable shear --locked`

#### License compliance, security advisories, other crate-level policy decisions (requires cargo-deny to be installed)

- `cargo deny check`

Treat all `cargo deny check` output as a blocking failure regardless of whether it is reported as an error or warning. Do not suggest adding deny exceptions without user confirmation.

## High-level architecture

- This is a Rust workspace with five crates:
  - `brot3-lib`: fractal engine, shared GPU/host data structures, shader entrypoints, and save-state types
  - `brot3-ui`: CLI entrypoint, egui/wgpu controller, headless rendering, and save/load orchestration
  - `shader_builder`: thin wrapper around `spirv-builder` that compiles `brot3-lib` to SPIR-V
  - `easy-shader-runner`: reusable framework for running the shader with winit/egui/wgpu, with optional runtime compilation
  - `xtask`: release helpers such as changelog and Debian packaging tasks

- `brot3-lib` is intentionally shared between host and GPU code. `lib/src/lib.rs` exposes the SPIR-V vertex/fragment/compute entrypoints, while `lib/src/data` defines the structs and enums that cross the host/shader boundary and `lib/src/engine` contains the actual fractal and colouring logic.
- Host-only support code lives under `lib/src/ui`, `lib/src/bignum`, and other `#[cfg(not(spirv))]` paths so it can be unit-tested without paying the cost of rebuilding the shader pipeline every time.
- `brot3-ui` has two top-level execution paths in `ui/src/lib.rs`: GUI mode and headless render mode. GUI mode instantiates `controller::Controller`, which derives `FragmentConstants` from `UiState` each frame and hands them to `easy-shader-runner`; headless mode loads a saved `UiState`, computes perturbation reference points if needed, renders via CPU or compute shader, and writes a PNG.
- Shader compilation is wired through `ui/build.rs`.
  1. Normal local build: leave `BROT3_PREBUILT_SHADER` unset. `ui/build.rs` runs `cargo run --release -p shader_builder` and embeds the resulting shader with `include_bytes!(env!("brot3_lib.spv"))`.
  2. Reproduce the CI shader path locally: set `BROT3_PREBUILT_SHADER` to the path of a previously built `.spv` file, then run `cargo build -p brot3-ui`.
  3. CI: `BROT3_PREBUILT_SHADER` is set by the pipeline to a previously built `.spv` artifact.

## Key conventions

- Treat `brot3_lib::ui::UiState` as the canonical saveable/user-facing state. The JSON save format is `UiStateSaveFile`, and PNG exports embed the same state in a `uistate` text chunk so `UiState::load_magic` can load either JSON files or PNGs with embedded state.
- When adding data that both the UI and shader need, extend the shared types in `lib/src/data` (`FragmentConstants`, `Flags`, `Palette`, enums) instead of creating parallel UI-only representations.
- Keep the host/SPIR-V split intact. Anything compiled for the shader must stay compatible with the `spirv` target and avoid host-only dependencies; host utilities belong behind `#[cfg(not(spirv))]`.
- Default app builds use features `ui` and `use-compiled-tools`. `hot-reload-shader` is a special workflow: it depends on runtime shader compilation and is expected to be paired with either `use-compiled-tools`, `use-installed-tools`, or an explicit `--spirv-tools` path. When suggesting `hot-reload-shader` usage, recommend `use-installed-tools` by default. Only switch that recommendation if the user explicitly states a different feature-flag preference or mentions a specific build environment such as compiled tools or a custom `spirv-tools` path.
- Tests are concentrated in `brot3-lib`. If you change engine logic, shared data layouts, zoom/state serialization, or colour math, add or update lib tests rather than relying on UI coverage.
