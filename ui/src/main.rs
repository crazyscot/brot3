//! User interface for brot3

mod components;
use brot3_engine::fractal::{Algorithm, Point, Scalar};
use components::{MainUI, Tile};
mod engine;
mod info;
mod types;
use types::{
    PixelCoordinate, PixelIndex, TileCoordinate, TileIndex, ZoomLevel, UI_TILE_SIZE,
    UI_TILE_SIZE_LOG2,
};

use brot3_engine::util::build_info;

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, VecModel};
use std::collections::BTreeMap;
use std::rc::Rc;

const UI_MAX_SEGMENT_SIZE: PixelIndex = 1 << (f32::MANTISSA_DIGITS - 1);
const UI_MAX_ZOOM_LEVEL: ZoomLevel = 45; // !!! Must match slider max in mainui.slint

/// Tile source, tiles, and active viewport parameters
struct World {
    loaded_tiles: BTreeMap<TileCoordinate, slint::Image>,
    loading_tiles: BTreeMap<TileCoordinate, Pin<Box<dyn Future<Output = slint::Image>>>>,
    /// Currently displayed zoom level
    zoom_level: ZoomLevel,
    /// Current size of the slint component, in pixels
    visible_height: PixelIndex,
    /// Current size of the slint component, in pixels
    visible_width: PixelIndex,

    // CAUTION: Slint's 'length' type is f32, which imposes a limit on the number of pixels it can reason about.
    // We work around this with a two-stage addressing scheme.
    // The WORLD is everything that could ever be shown (all possible tiles)
    // The SEGMENT is the slint draggable region, identified by segment_origin_{x,y}
    // The OFFSET is the currently visible portion in the viewport, identified by segment_offset_{x,y} relative to segment_origin.
    /// The leftmost visible pixel of the segment that is currently bound to the viewport
    segment_offset_x: PixelIndex,
    /// The topmost visible pixel of the segment that is currently bound to the viewport
    segment_offset_y: PixelIndex,
    /// The origin of the visible (draggable) segment relative to the world
    segment_origin_x: PixelIndex,
    /// The origin of the visible (draggable) segment relative to the world
    segment_origin_y: PixelIndex,

    /// Co-ordinates of the bottom-left-most tile currently rendered (caution, may not be visible)
    bottom_left_tile: TileCoordinate,
}

impl World {
    fn new() -> Self {
        World {
            loaded_tiles: BTreeMap::default(),
            loading_tiles: BTreeMap::default(),
            zoom_level: 1,
            visible_height: 0,
            visible_width: 0,
            // The offset is the current visible position in the viewport, relative to the segment origin.
            segment_offset_x: 0,
            segment_offset_y: 0,
            // The segment is the region of the universe that the viewport can drag around.
            segment_origin_x: 0,
            segment_origin_y: 0,

            bottom_left_tile: TileCoordinate { z: 0, x: 0, y: 0 },
        }
    }

    /// Actions a zoom in/out request
    /// * `zoom_level`: New zoom level
    /// * `ox`: X coordinate of the zoom centre, in current-zoom-level pixel coordinates
    /// * `oy`: Y coordinate of the zoom centre, in current-zoom-level pixel coordinates
    fn set_zoom_level(&mut self, zoom_level: ZoomLevel, ox: PixelIndex, oy: PixelIndex) {
        if self.zoom_level != zoom_level {
            self.loaded_tiles.clear();
            self.loading_tiles.clear();
            // TODO: Can we abort loading tiles we no longer care about? Perhaps some sort of handle (or just a flag?) goes into the BTreeMap value.

            // Apply the zoom to compute our new offset x and y
            #[allow(clippy::cast_possible_wrap)]
            let bit_shift = (zoom_level - self.zoom_level) as isize;
            let mut world_offset_x = self.offset_to_world_x() + ox;
            let mut world_offset_y = self.offset_to_world_y() + oy;
            if bit_shift > 0 {
                world_offset_x <<= bit_shift;
                world_offset_y <<= bit_shift;
            } else {
                world_offset_x >>= -bit_shift;
                world_offset_y >>= -bit_shift;
            }

            world_offset_x -= ox;
            world_offset_y -= oy;

            self.reset_segment(zoom_level, world_offset_x, world_offset_y);
            self.reset_view();
        }
    }

    /// Rearrange the segment around a given point.
    /// Caller is responsible for managing loaded/loading tiles.
    /// `world_offset_x` and `world_offset_y` are pixel coordinates relative to the world.
    fn reset_segment(
        &mut self,
        zoom_level: ZoomLevel,
        world_offset_x: PixelIndex,
        world_offset_y: PixelIndex,
    ) {
        // Decompose the offset into segment origin and segment offset.
        let segment_size = World::segment_size_for(zoom_level);
        self.segment_origin_x = std::cmp::max(world_offset_x - (segment_size / 2), 0);
        self.segment_origin_y = std::cmp::max(world_offset_y - (segment_size / 2), 0);
        self.segment_offset_x = world_offset_x - self.segment_origin_x;
        self.segment_offset_y = world_offset_y - self.segment_origin_y;
        self.zoom_level = zoom_level;
    }
    /// Special case wrapper for `reset_segment` which recentres the current view.
    /// Caution: This should only be called from `State.recentre_segment()`
    fn recentre_world_segment(&mut self) {
        let world_offset_x = self.segment_origin_x + self.segment_offset_x;
        let world_offset_y = self.segment_origin_y + self.segment_offset_y;
        println!("Recentring around x={world_offset_x} y={world_offset_y} z={}; offset x={}, offset y={} ", self.zoom_level, self.segment_origin_x, self.segment_origin_y);
        self.reset_segment(self.zoom_level, world_offset_x, world_offset_y);

        // Self check: If we recompute world_offset_x and y now, they haven't changed.
        let new_x = self.segment_origin_x + self.segment_offset_x;
        let new_y = self.segment_origin_y + self.segment_offset_y;
        println!(
            "New x={new_x} y={new_y}; segment offsets: x={}, y={} ",
            self.segment_origin_x, self.segment_origin_y
        );
        if new_x != world_offset_x {
            eprintln!("ERROR: X recentre mismatch {world_offset_x} -> {new_x}");
        }
        if new_y != world_offset_y {
            eprintln!("ERROR: Y recentre mismatch {world_offset_y} -> {new_y}");
        }
    }

    fn world_size_for(zoom_level: ZoomLevel) -> PixelIndex {
        PixelIndex::from(UI_TILE_SIZE) * (1 << zoom_level)
    }
    fn segment_size_for(zoom_level: ZoomLevel) -> PixelIndex {
        std::cmp::min(World::world_size_for(zoom_level), UI_MAX_SEGMENT_SIZE)
    }

    /// Updates the view on startup or after a user action.
    /// Launches tile loads as necessary.
    fn reset_view(&mut self) {
        /// How many cached tiles to keep around the currently-visible set?
        /// IOW: How far away does a tile need to be from the viewport in order to be dropped?
        const KEEP_CACHED_TILES: TileIndex = 10;

        let m = 1 << self.zoom_level; // max number of tiles in either dimension

        // Compute currently visible tile range
        let offset_x = self.offset_to_world_x();
        let offset_y = self.offset_to_world_y();
        let min_x = offset_x >> UI_TILE_SIZE_LOG2;
        let min_y = offset_y >> UI_TILE_SIZE_LOG2;
        let max_x = (((offset_x + self.visible_width + 1) >> UI_TILE_SIZE_LOG2) + 1).min(m);
        let max_y = (((offset_y + self.visible_height + 1) >> UI_TILE_SIZE_LOG2) + 1).min(m);

        // Remove tiles that are too far away
        let keep = |coord: &TileCoordinate| {
            coord.z == self.zoom_level
                && (coord.x > min_x - KEEP_CACHED_TILES)
                && (coord.x < max_x + KEEP_CACHED_TILES)
                && (coord.y > min_y - KEEP_CACHED_TILES)
                && (coord.y < max_y + KEEP_CACHED_TILES)
        };
        self.loading_tiles.retain(|coord, _| keep(coord));
        self.loaded_tiles.retain(|coord, _| keep(coord));

        for y in min_y..max_y {
            for x in min_x..max_x {
                let coord = TileCoordinate {
                    z: self.zoom_level,
                    x,
                    y,
                };
                if self.loaded_tiles.contains_key(&coord) {
                    continue;
                }

                // forcibly bind the future to a variable, we care that it happens
                let _a = self.loading_tiles.entry(coord).or_insert_with(|| {
                    Box::pin(async move {
                        let (send, recv) = tokio::sync::oneshot::channel();
                        rayon::spawn(move || {
                            let _ = send.send(engine::draw_tile(&coord));
                        });
                        let buffer = recv.await.unwrap().unwrap_or_else(|e| {
                            eprintln!("error drawing tile: {e}");
                            SharedPixelBuffer::<Rgba8Pixel>::new(1, 1)
                        });
                        slint::Image::from_rgba8(buffer)
                    })
                });
            }
        }
        self.bottom_left_tile = TileCoordinate {
            z: self.zoom_level,
            x: min_x,
            y: max_y,
        }
    }

    /// Checks the `loading_tiles` set for tiles which have become ready.
    /// Moves any ready tiles into the `loaded_tiles` list.
    /// * `context`: polling context
    /// * `changed`: (out) Will be set to true if any tiles were
    fn poll(&mut self, context: &mut Context<'_>, changed: &mut bool) {
        self.loading_tiles.retain(|coord, future| {
            let image = future.as_mut().poll(context);
            match image {
                Poll::Ready(image) => {
                    let _ = self.loaded_tiles.insert(*coord, image);
                    *changed = true;
                    false
                }
                Poll::Pending => true,
            }
        });
    }

    /// Converts a pixel address within the draggable segment to a pixel address in the world
    fn offset_to_world_x(&self) -> PixelIndex {
        self.segment_origin_x + self.segment_offset_x
    }
    /// Converts a pixel address within the draggable segment to a pixel address in the world
    fn offset_to_world_y(&self) -> PixelIndex {
        self.segment_origin_y + self.segment_offset_y
    }
    /// Returns the address of the top-left-most currently visible pixel of the world
    fn visible_origin(&self) -> PixelCoordinate {
        PixelCoordinate {
            x: self.offset_to_world_x(),
            y: self.offset_to_world_y(),
        }
    }

    /// Current visible size, in pixels
    fn visible_dimensions(&self) -> PixelCoordinate {
        PixelCoordinate {
            x: self.visible_width,
            y: self.visible_height,
        }
    }
    /// Size of the whole world, in pixels (in either dimension; it's square)
    fn world_size(&self) -> PixelIndex {
        World::world_size_for(self.zoom_level)
    }
}

/// Top-level program state
struct State {
    /// The world we are displaying
    world: RefCell<World>,
    /// The UI component we are updating
    main_ui: MainUI,
    /// Polling handle
    poll_handle: RefCell<Option<slint::JoinHandle<()>>>,
}

impl State {
    /// Polls for newly-loaded tiles so they can be displayed
    fn do_poll(self: Rc<Self>) {
        if let Some(handle) = self.poll_handle.take() {
            handle.abort();
        }
        self.refresh_model();
        let _a = slint::spawn_local(async move {
            std::future::poll_fn(|context| {
                let mut changed = false;
                self.world.borrow_mut().poll(context, &mut changed);
                if changed {
                    self.refresh_model();
                }
                if self.world.borrow().loading_tiles.is_empty() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
        })
        .unwrap();
    }

    /// Pushes the loaded tiles and their data to slint
    fn refresh_model(&self) {
        #![allow(clippy::cast_precision_loss)]
        let world = self.world.borrow();
        let segment_origin_x = world.segment_origin_x;
        let segment_origin_y = world.segment_origin_y;
        drop(world);
        let vec = VecModel::from(
            self.world
                .borrow()
                .loaded_tiles
                .iter()
                .map(|(coord, image)| Tile {
                    tile: image.clone(),
                    // Tile coordinates passed to slint are relative to the segment origin
                    x: (coord.x * PixelIndex::from(UI_TILE_SIZE) - segment_origin_x) as f32,
                    y: (coord.y * PixelIndex::from(UI_TILE_SIZE) - segment_origin_y) as f32,
                })
                .collect::<Vec<Tile>>(),
        );
        self.main_ui.set_tiles(slint::ModelRc::new(vec));
        self.update_info();
    }

    fn update_info(&self) {
        self.main_ui.set_algorithm("Original".into()); // TODO this comes from alg spec
        self.main_ui.set_colourer("LogRainbow".into()); // TODO from alg spec
        self.main_ui.set_max_iter(types::UI_TEMP_MAXITER.into()); // TODO from alg spec

        let world = self.world.borrow();
        let window_dimensions = world.visible_dimensions();
        let world_size_pixels = world.world_size();
        let algorithm_instance =
            brot3_engine::fractal::factory(brot3_engine::fractal::Selection::Original); // TODO use algorithm from spec
        let fractal_size = algorithm_instance.default_axes();

        #[allow(clippy::cast_precision_loss)]
        // window_dimensions is small (screen resolution) so no precision loss.
        // world_size_pixels is a power of 2 so no precision loss.
        let visible_axes_length = brot3_engine::fractal::Point::new(
            window_dimensions.x as Scalar * fractal_size.re / world_size_pixels as Scalar,
            window_dimensions.y as Scalar * fractal_size.im / world_size_pixels as Scalar,
        );

        #[allow(clippy::cast_precision_loss)]
        // world_size_pixels is a power of 2 so no precision loss.
        let complex_pixel_size: Point = fractal_size.unscale(world_size_pixels as Scalar);

        let top_left_pixel = world.visible_origin();
        let bottom_left_pixel = PixelCoordinate {
            x: top_left_pixel.x,
            y: top_left_pixel.y + world.visible_height - 1,
        };
        // Location of the bottom left pixel, expressed as a vector relative to the bottom left of the world
        let bottom_left_offset: PixelCoordinate = PixelCoordinate {
            x: top_left_pixel.x,
            y: world.world_size() - bottom_left_pixel.y - 1,
        };
        // Offset of the bottom left pixel, in complex units, from the fractal origin
        #[allow(clippy::cast_precision_loss)]
        // Maximum pixel size, and hence bottom_left_offset, are limited to fit within f64 mantissa (TECHDEBT)
        let origin_offset = Point::new(
            complex_pixel_size.re * bottom_left_offset.x as Scalar,
            complex_pixel_size.im * bottom_left_offset.y as Scalar,
        );
        let origin_absolute = origin_offset - algorithm_instance.default_axes().unscale(2.)
            + algorithm_instance.default_centre();

        let axes_precision = info::axes_precision_for_canvas(window_dimensions);
        let real_axis =
            info::format_float_with_precision("", visible_axes_length.re, axes_precision);
        let imag_axis =
            info::format_float_with_precision("+", visible_axes_length.im, axes_precision);
        let axes = format!("{real_axis}{imag_axis}i");

        let position_dp = info::decimal_places_for_axes(window_dimensions, visible_axes_length);
        let origin_real = info::format_float_fixed("", origin_absolute.re, position_dp);
        let origin_imag = info::format_float_fixed("+", origin_absolute.im, position_dp);
        let origin = format!("{origin_real}{origin_imag}i");

        self.main_ui.set_origin(origin.into());
        self.main_ui.set_axes(axes.into());

        // N.B. zoom is already wired up in slint to the zoom control, so we don't need to set it here
    }

    /// Updates the viewport after the zoom or segment origin changes.
    /// This is always called after `set_zoom_level` or `recentre_segment`, which update the world's segment origin and offset.
    fn set_viewport_size(&self) {
        #![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let world = self.world.borrow();
        let zoom = world.zoom_level;
        self.main_ui.set_zoom(zoom as _);
        let segment_size = World::segment_size_for(zoom);
        self.refresh_model();
        self.main_ui.invoke_set_viewport(
            -world.segment_offset_x as f32,
            -world.segment_offset_y as f32,
            segment_size as f32,
            segment_size as f32,
        );
    }

    /// Recentres the segment around the current viewport.
    /// This works around slint's inherent limitation, caused by lengths being f32.
    fn recentre_segment(&self) {
        let mut world = self.world.borrow_mut();
        world.recentre_world_segment();
        // There doesn't seem to be any need to call world.reset_view() here.
        drop(world);
        self.set_viewport_size();
    }
}

fn main() {
    #![allow(trivial_numeric_casts)]

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    let state = Rc::new(State {
        world: RefCell::new(World::new()),
        main_ui: MainUI::new().unwrap(),
        poll_handle: None.into(),
    });

    state
        .main_ui
        .set_window_title(format!("brot3 {}", build_info::PKG_VERSION).into());

    let state_weak = Rc::downgrade(&state);
    #[allow(clippy::cast_possible_truncation)]
    // User flicked (dragged) the map
    // * ox: New absolute viewport offset
    // * oy: New absolute viewport offset
    state.main_ui.on_flicked(move |ox, oy| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        world.segment_offset_x = -ox as _;
        world.segment_offset_y = -oy as _;
        world.visible_width = state.main_ui.get_visible_width() as _;
        world.visible_height = state.main_ui.get_visible_height() as _;
        world.reset_view();
        drop(world); // drops the reference, not the actual world
        state.do_poll();
    });

    let state_weak = Rc::downgrade(&state);
    #[allow(clippy::cast_possible_truncation)]
    // User dragged the zoom slider. Change zoom without panning i.e. keep the centre where it is.
    // * zoom: New zoom level
    state.main_ui.on_zoom_changed(move |zoom| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        world.visible_width = state.main_ui.get_visible_width() as _;
        world.visible_height = state.main_ui.get_visible_height() as _;
        let (vw, vh) = (world.visible_width, world.visible_height);
        world.set_zoom_level(zoom as _, vw / 2, vh / 2);
        drop(world); // drops the reference, not the actual world
        state.set_viewport_size();
        state.do_poll();
    });
    let state_weak = Rc::downgrade(&state);
    #[allow(clippy::cast_possible_truncation)]
    // User gestured to zoom in. Zoom around a given point.
    // * ox: Zoom locus, X coordinate?
    // * oy: Zoom locus, Y coordinate
    state.main_ui.on_zoom_in(move |ox, oy| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        let z = (world.zoom_level + 1).min(UI_MAX_ZOOM_LEVEL);
        world.visible_width = state.main_ui.get_visible_width() as _;
        world.visible_height = state.main_ui.get_visible_height() as _;
        world.set_zoom_level(z as _, ox as PixelIndex, oy as PixelIndex);
        drop(world); // drops the reference, not the actual world
        state.set_viewport_size();
        state.do_poll();
    });
    let state_weak = Rc::downgrade(&state);
    #[allow(clippy::cast_possible_truncation)]
    // User gestured to zoom out. Zoom around a given point.
    // * ox: Zoom locus, X coordinate?
    // * oy: Zoom locus, Y coordinate
    state.main_ui.on_zoom_out(move |ox, oy| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        let z = (world.zoom_level - 1).max(1);
        world.visible_width = state.main_ui.get_visible_width() as _;
        world.visible_height = state.main_ui.get_visible_height() as _;
        world.set_zoom_level(z as _, ox as PixelIndex, oy as PixelIndex);
        drop(world); // drops the reference, not the actual world
        state.set_viewport_size();
        state.do_poll();
    });

    // Recentre the segment around the viewport visible region.
    // TODO TECHDEBT: Do this automatically, if we can figure out how to get notified by slint when a drag episode ends.
    // (Potentially triggering on every drag event is overkill. Such events are raised both during and after the drag episode.
    // Modifying the viewport during a drag episode [button still pressed] doesn't work properly;
    // we need to be able to tell that there is no current drag event. Perhaps we set a recentre-needed flag during the drag,
    // then on poll we check the mouse button state and jump in after it has been released?)
    let state_weak = Rc::downgrade(&state);
    state.main_ui.on_resegment_clicked(move || {
        let state = state_weak.upgrade().unwrap();
        state.recentre_segment();
    });

    {
        let state = state.clone();
        #[allow(clippy::cast_possible_truncation)]
        let _a = slint::spawn_local(async move {
            let mut world = state.world.borrow_mut();
            world.visible_width = state.main_ui.get_visible_width() as _;
            world.visible_height = state.main_ui.get_visible_height() as _;
            world.reset_view();
            drop(world); // drops the reference, not the actual world
            state.set_viewport_size();
            state.clone().do_poll();
        })
        .unwrap();
    }

    state.main_ui.run().unwrap();
}
