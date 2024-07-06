# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2024-07-06

### <!-- 0 -->🚀 Features

- *(colouring)* Add SqrtRainbow colourer
- *(ui)* Tweak navigator background to match main image background
- *(ui)* Set placeholder fill style
- *(ui)* Compute home position margins to make the fractal fully visible
- *(ui)* Add busy indicator ([#121](https://github.com/crazyscot/brot3/issues/121))
- Allow brot3 to be an all-in-one binary that also accepts CLI arguments


### <!-- 1 -->🐛 Bug Fixes

- *(engine)* Make all misc_fractals square, tweak default centre points for aesthetics
- *(engine)* Avoid numeric precision issue in some fractal combinations
- *(ui)* Save Image workflow uses current selected colourer
- *(ui)* Ignore fleeting zero-axes situations when updating the HUD
- *(ui)* Don't propagate exceptions from updateIndicator
- *(ui)* Show Origin, not centre, by default for consistency
- *(ui)* Preserve position when changing max_iter
- *(ui)* Preview menus use current max_iter, up to a point
- *(ui)* Allow zooming out well beyond 1:1 pixels
- *(ui)* Improve navigator region colouring, for better visibility against most fractals
- *(ui)* Make colour cycling robust against user leaning on one of the shortcut combos.
- *(ui)* When changing source, defer updating the HUD until after the metadata have loaded.
- *(ui)* Explicitly request the desired image type on data URLs
- Go To Position form now works correctly at all window aspect ratios


### <!-- 2 -->🚜 Refactor

- *(cli)* Split up into a library and binary crate
- *(ui)* Add promise for the metadata in EngineTileSource
- *(ui)* Remove unnecessary noop functions
- *(ui)* Cope with FractalView arriving as a plain dict (from tauri), not as a blessed object


### <!-- 3 -->📚 Documentation

- Add an exemplar render
- Set up git-cliff, autogenerate changelog to date ([#120](https://github.com/crazyscot/brot3/issues/120))


### <!-- 4 -->⚡ Performance

- *(ui)* Remove unnecessary async-mutex in the global serial allocator
- Tweak tile sizes


### <!-- 5 -->🎨 Styling

- Update icon
- Better fonts; rework HUD; consolidate menus


### <!-- 7 -->⚙️ Miscellaneous Tasks

- *(ui)* Tidy up questionable uses of non-null-assertion operator
- Add vite react plugin to silence a load of noise at build-time
- Yarn add @types/node --dev to silence some warnings
- Silence chunk size warning
- Yarn upgrade
- Cargo update clap strum_macros tauri


### Cargo

- *(deps)* Bump the cargo group with 4 updates


### Npm

- *(deps)* Bump the npm group in /ui with 4 updates


## [0.9.4] - 2024-06-05

### Cargo

- *(deps)* Bump the cargo group with 2 updates
- *(deps)* Bump the cargo group with 3 updates


### Cli

- Add option to print the info string


### Engine

- Introduce float_format utilities
- Update plot info strings to output with the same precision rules as the HUD


### Npm

- *(deps-dev)* Bump vite from 5.2.11 to 5.2.12 in /ui in the npm group


### Ui

- Apply precision bounds in HUD, format floats more consistently


## [0.9.2] - 2024-03-21

### Menu.rs

- Wrap functions into a struct


### Rust

- Add get_metadata, add ts twins


## [0.9.1-unreleased] - 2024-03-08

### <!-- 7 -->⚙️ Miscellaneous Tasks

- On macos, build universal binary


### Cli

- Add missing help text, --help aliases


### Render_rgba

- Remove a middleman variable


## [0.9.0-unreleased] - 2024-02-29

### Tidyup

- Move type definitions to their own source


### Tilespec

- Tidy up accessors, add top_left & bottom_right quasi-accessors


### Viewertilespec

- Fields should be u64, add Display trait


## [0.1.0] - 2024-02-03

### Renderer

- Refactor render to be render-to-file, taking filename & colourer.


### Tile

- :new now takes PlotData
- :join()


### Bugfix

- Origin is bottom-left; output the top row first
- Refactor data reversal in y axis


### Cli

- Move styling out to a module
- Split and rejoin the tile (still single threaded for now)
- Print the info string
- Split out file extension autodetector


### Cli/plot

- Tests


### Colourer

- Refactor max_iters as u32, for consistency


### Colouring

- :framework needn't be a pub mod
- Move all types into 'types'
- Move Rgb8, OutputsHsvf from types into framework
- (what's left of) types.rs -> testing.rs
- Use from_color_unclamped


### Filename

- Get a Path properly
- Write_handle gives a buffered handle
- Add open_for_writing helper


### Fractal

- Move framework out to its own source file, for consistency
- Move maths types out to their own source


### Get_test_tile_spec

- Now takes a dimension argument


### Listable

- Add list2() for now


### Png

- Refactor render into two
- Make -o- work


### Pointdata

- Refactor to use ndarray, which has mutable slice accessors


### Render

- :factory: reorder ctor arguments for consistency
- Move framework to its own file, for consistency
- Tidy up derives
- Use enum_delegate instead of enum_dispatch
- Replace EnumIter with a longer-winded String pathway


### Tilespec

- Store width & height as a tuple
- Use non-square test data for split tests


<!-- generated by git-cliff -->
