## [3.0.0] - 2025-10-20

### 🚀 Features

- Initial mandelbrot shader
- *(ui)* Basic keyboard and mouse-driven movement
- *(ui)* Show rendering fps & fractal coordinates
- *(fractal)* Add smoothed escape count
- Controllable iteration limit
- *(ui)* Hide/show the co-ordinates and FPS overlays
- Add algorithm selection to ui; add Mandeldrop fractal
- *(maths)* Add Mandelbar
- *(maths)* Port the BurningShip, Celtic, Variant BirdOfPrey algorithms from brot3_enging
- *(ui)* Vsync checkbox
- Add integer exponent as a parameter
- Exponent can be a float
- *(util)* HSL colour conversion
- Add SqrtRainbow colourer, selection UI
- Port some colourers from previous brot3 branches
- Configurable palettes
- Add Monochrome palette
- Offset range becomes -10 .. 10
- Add gamma control (currently only to Monochrome palette)
- *(test)* Shader benchmarking
- *(keyboard)* Add Ctrl-Q and T
- *(keyboard)* Add [,] to cycle through palettes
- *(keyboard)* Add gradient, offset, gamma, saturation, lightness
- *(ui)* Keyboard help window
- Make fractional iterations optional
- *(ui)* Draggable scale bar
- LCH gradient
- Support variable runtime compilation directory paths
- *(cli)* Add --spirv-tools
- *(cli)* Add --static-shader
- *(ui)* Fullscreen option, keyboard control & CLI
- *(ui)* F1 toggles keyboard help
- *(ui)* Add --no-ui; show/hide UI moves to F2
- *(ui)* Palette keys move to F5/F6
- *(cli)* Add --colourer
- *(ui)* Fractal selection via CLI and keyboard

### 🐛 Bug Fixes

- Celtic
- *(shader)* Correct smoothed iteration count algorithm
- Apply palette numeric limits to keyboard controls

### 💼 Other

- Hello shader world using abel's framework
- Disable tests on all crates, for now; add shared to top-level manifest
- Rename builder to shader_builder, add shader build to CI
- Import big_complex and big_vec2 from abel465
- *(ui)* Adjust initial complex position
- De-trait & de-structure the colouring algorithms
- *(ui)* Put exponent & palette detail into collapsibles
- *(ui)* Add about/license dialogs
- Handle errors from easy_shader_runner
- Use cfg_aliases to simplify
- Set up cargo-deb metadata & desktop application entry
- Include changelogs in debian packaging
- Regenerate build info when the git commit HEAD changes

### 🚜 Refactor

- Rearrange crates -> engine3/[runner,shader,shared]; ui3
- Extract Grid, GridRef, GridRefMut into a new shader_util crate
- Move Bool and Size types into shader_util
- Simplify structure of shared crate, rename to engine_common
- Combine code for Mandelbrot and Mandelbar via traits
- *(shader)* Monomorphise exponentiation, move the matcher out of the inner loop
- *(shader)* Rename Fractal::modify
- *(shader)* Move iterate_algorithm into Fractal
- *(shader)* Move pre_modify_point into Fractal
- *(shader)* Ditch now-empty Modifier trait
- *(shader)* Rename Fractal to FractalImpl
- *(shader)* Move common fractal data into Builder
- *(shader)* Merge Builder into FractalIterator
- *(shader)* Merge RenderData and FractalResult -> PointResult
- *(shader)* Merge iterations and iteration_loop
- *(shader)* Rename FractalRunner, FractalImpl
- *(shader)* Move lograinbow into the beginning of a colouring framework

### 🧪 Testing

- Create initial unit tests for shader
- Improve coverage

### ⚙️ Miscellaneous Tasks

- *(build)* Remove rust-analyzer path from vscode settings
- *(build)* Set up toolchain override for rust-gpu
- Highlight hard-wired paths
- Set up license & authors on builder & shared crates
- Fix clippy warning
- Disable engine builds for now
- Re-enable unit tests
- Rename runner crate to brot3-ui, binary to brot3
- Fix crate versions, descriptions
- Move/rename shader and common crates
- Split out ui and shader jobs, rename clippy job
- *(test)* Fix html prefixing in coverage report
- Remove old engine & cli directories
- Update egui to 0.33 (synchronised with easy-shader-runner)
- Update rust-gpu (synchronised with easy-shader-runner)
- Use PathBuf more consistently
- Handle shader build failures more tidily
- Rename ui3 to ui
- Make enum increment in shader_util a macro
- Revise CHANGELOG autogeneration config
## [2.0.0-pre] - 2025-09-16

### 🚀 Features

- *(ui)* [**breaking**] Rework GUI as a basic tiled view using slint
- *(ui)* Basic HUD info display (#143)
- *(ui)* Apply constraints to resize and zoom events
- *(ui)* Abandon outstanding renders when they are no longer needed
- *(ui)* Show zoom in a more human-friendly format
- *(ui)* Add centre to info panel
- *(ui)* Function menu and About box
- *(ui)* Menu option to show/hide info panel
- *(ui)* Fractal & Colourer selection via ComboBox

### 🐛 Bug Fixes

- *(ci)* Use dev profile in the benchmark checks

### 💼 Other

- Don't force non-standard linker
- Rust 1.89, 2024 edition

### 🚜 Refactor

- Fractal::decode & colouring::decode should be FromStr
- *(engine)* Pivot Render framework to spire_enum, refactor Listable to suit
- *(engine)* Pivot Colourer to spire_enum
- *(engine)* Pivot Algorithm to spire_enum

### ⚡ Performance

- *(ui)* Clear loading tiles on fractal/colourer change
- *(ui)* Keep plotted tile data and reuse when appropriate

### 🎨 Styling

- Set a default ui font
- Add a gradient background under the fractal area
- Move info display to top of window, button sits beneath it
- Use vivi for the menu button
- Use vivi comboboxes for the drop-downs
- Rework Info box layout
- Reposition info box & menu button

### ⚙️ Miscellaneous Tasks

- Differentiate CI jobs by branch
- *(ci)* Update dependabot config, and cliff to suit
- *(ci)* Add iai benchmarker to CI, fix it
- Update to rust 1.80 (#144)
- *(ci)* Tidy up CI rules (#145)
- *(ci)* Binstall --no-confirm
- *(ci)* Don't use matrix where we don't need to
- Drop defunct tile_cache module
- *(ci)* Create branch cache cleanup action
- Refactor tile loading out into its own type & module
- *(ci)* Consolidate benchmark check to simplify CI
- *(engine)* Add missing Ord and PartialOrd derives to fractals, colourers and AlgorithmSpec
- Add AlgorithmSpec to TileCoordinates
- *(ui)* Cache shared strings to reduce memory churn
- Refactor gesture setup into a separate function
- Move menu setup into that module
- Make TileSpec Defaultable
- Use TextBase inheritance to reduce repetition
- *(ci)* Force explicit iai runner version
- *(ci)* Don't run rustup update needlessly
- *(ci)* Query cargo for the version of iai-callgrind-runner to install
- *(ci)* Update build platform, improve cleanup action, remove defunct release workflow
- *(ci)* Fix slint deprecation warning
- *(ci)* Linter fixes for toolchain 1.86
- *(skip,ci,deps)* Pin the version of iai-callgrind to prevent CI failures when it changes
- *(ci)* Turn down github action updates
- *(ci)* Review ci, split up jobs
- Refactor away most uses of dyn
- Add coverage analysis task
- More tweaks
## [1.0.0] - 2024-07-06

### 🚀 Features

- *(ui)* Tweak navigator background to match main image background
- *(ui)* Set placeholder fill style
- *(ui)* Compute home position margins to make the fractal fully visible
- *(ui)* Add busy indicator (#121)
- *(colouring)* Add SqrtRainbow colourer
- Allow brot3 to be an all-in-one binary that also accepts CLI arguments

### 🐛 Bug Fixes

- *(ui)* Save Image workflow uses current selected colourer
- *(ui)* Ignore fleeting zero-axes situations when updating the HUD
- *(ui)* Don't propagate exceptions from updateIndicator
- *(ui)* Show Origin, not centre, by default for consistency
- *(ui)* Preserve position when changing max_iter
- *(ui)* Preview menus use current max_iter, up to a point
- *(ui)* Allow zooming out well beyond 1:1 pixels
- *(ui)* Improve navigator region colouring, for better visibility against most fractals
- *(engine)* Make all misc_fractals square, tweak default centre points for aesthetics
- *(engine)* Avoid numeric precision issue in some fractal combinations
- *(ui)* Make colour cycling robust against user leaning on one of the shortcut combos.
- *(ui)* When changing source, defer updating the HUD until after the metadata have loaded.
- *(ui)* Explicitly request the desired image type on data URLs
- Go To Position form now works correctly at all window aspect ratios

### 💼 Other

- *(deps)* Bump the npm group in /ui with 4 updates
- *(deps)* Bump the cargo group with 4 updates
- Fix OSX license path (#130)

### 🚜 Refactor

- *(ui)* Add promise for the metadata in EngineTileSource
- *(ui)* Remove unnecessary noop functions
- *(ui)* Cope with FractalView arriving as a plain dict (from tauri), not as a blessed object
- *(cli)* Split up into a library and binary crate

### 📚 Documentation

- Add an exemplar render
- Set up git-cliff, autogenerate changelog to date (#120)
- Improve cliff config, regenerate CHANGELOG

### ⚡ Performance

- *(ui)* Remove unnecessary async-mutex in the global serial allocator
- Tweak tile sizes

### 🎨 Styling

- Update icon
- Better fonts; rework HUD; consolidate menus
- Change MaxIter accelerator to avoid clashing with Cmd-M on OSX (#131)

### ⚙️ Miscellaneous Tasks

- *(ui)* Tidy up questionable uses of non-null-assertion operator
- Add vite react plugin to silence a load of noise at build-time
- Yarn add @types/node --dev to silence some warnings
- Silence chunk size warning
- Yarn upgrade
- Cargo update clap strum_macros tauri
- Tweak github release template
- Tweak dependabot config (commit messages)
## [0.9.4] - 2024-06-05

### 🚀 Features

- *(ui)* Apply precision bounds in HUD, format floats more consistently
- *(cli)* Add option to print the info string
- *(engine)* Introduce float_format utilities
- *(engine)* Update plot info strings to output with the same precision rules as the HUD

### 💼 Other

- *(deps)* Bump the cargo group with 2 updates
- *(deps-dev)* Bump vite from 5.2.11 to 5.2.12 in /ui in the npm group
- *(deps)* Bump the cargo group with 3 updates

### ⚙️ Miscellaneous Tasks

- Fix (or possibly perpetrate?) build argument madness on OSX (#80)
- Update rust version for CVE (#86)
- Viewer: make fields private (#95)
## [0.9.3] - 2024-04-01

### 🚀 Features

- *(ui)* Go To Point (#73)
- *(ui)* Add centre to position display
- *(ui)* Add close X controls to HUD and position entry form
- *(ui)* Remove menu checkmark on Go To Position
- *(ui)* Add Copy Current Position button
- *(ui)* Rearrange HUD/goto form
- *(ui)* Add GenericError alerter
- Implement Save Image
- Allow user to specify the size to save at

### 🐛 Bug Fixes

- Defer initial position display

### 💼 Other

- Rework TileSpec::Display to be filename-friendly

### 🚜 Refactor

- Split updateIndicator for readability
- Move HUD and position entry forms into viewer.ts
- HUD selects element of the info panel, not whole document
- Rename variables & fields for consistency
- Rename go_to -> go_to_position for clarity; remove needless extra functions
- Move DOM utility classes out to separate file
- Move HUD class, HTML and functionality out to separate file
- Rename FractalMetadata => FractalView (rust & TS)
- Introduce the helper type UserDestination
- Have the menu builder clone and hack tauri os_default for now
- Promote autodetect_extension to render::framework
- Move GenericError to a new Util module
- Rename dom_util.tr_is_visible

### ⚙️ Miscellaneous Tasks

- Code formatting TS,CSS (#76)
- Ci fixes
## [0.9.2] - 2024-03-21

### 💼 Other

- Add get_metadata, add ts twins
- Wrap functions into a struct
## [0.9.1-unreleased] - 2024-03-08

### 🚀 Features

- *(cli)* Add missing help text, --help aliases

### 🐛 Bug Fixes

- List outputs item names in kebab-base, to match what we accept

### 💼 Other

- Remove a middleman variable

### ⚙️ Miscellaneous Tasks

- On macos, build universal binary
## [0.9.0-unreleased] - 2024-02-29

### 💼 Other

- Tidy up accessors, add top_left & bottom_right quasi-accessors
- Fields should be u64, add Display trait
- Move type definitions to their own source

### ⚙️ Miscellaneous Tasks

- Apt update before installing
- Use rust-toolchain action
- Tidy upworkflow
## [0.1.0] - 2024-02-03

### 🚀 Features

- *(cli)* Move styling out to a module
- *(cli)* Split and rejoin the tile (still single threaded for now)
- *(cli)* Print the info string
- *(cli)* Split out file extension autodetector

### 💼 Other

- Get a Path properly
- :new now takes PlotData
- Refactor render into two
- Make -o- work
- Write_handle gives a buffered handle
- Tests
- Origin is bottom-left; output the top row first
- Store width & height as a tuple
- Use non-square test data for split tests
- :join()
- Refactor data reversal in y axis
- :framework needn't be a pub mod
- :factory: reorder ctor arguments for consistency
- Move framework to its own file, for consistency
- Move all types into 'types'
- Move framework out to its own source file, for consistency
- Move maths types out to their own source
- Add open_for_writing helper
- Refactor render to be render-to-file, taking filename & colourer.
- Tidy up derives
- Use enum_delegate instead of enum_dispatch
- Add list2() for now
- Replace EnumIter with a longer-winded String pathway
- Move Rgb8, OutputsHsvf from types into framework
- (what's left of) types.rs -> testing.rs
- Now takes a dimension argument
- Refactor max_iters as u32, for consistency
- Use from_color_unclamped
- Refactor to use ndarray, which has mutable slice accessors
