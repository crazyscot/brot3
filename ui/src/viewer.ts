// brot3 fractal viewer (bridge to OpenSeadragon)
// (c) 2024 Ross Younger

import { invoke } from '@tauri-apps/api'
import { UnlistenFn, listen } from '@tauri-apps/api/event'
import jQuery from 'jquery'
import OpenSeadragon from 'openseadragon'

import { EnginePoint, FractalView, TileSpec, TileResponse, TileError, TilePostData, TileResponseHelper, ListItem } from './engine_types'
import { HeadsUpDisplay, UserDestination } from './hud'
import { nextSerial } from './serial_allocator'

export const TILE_SIZE = 128;
const IMAGE_DIMENSION = 1024 * 1024 * 1024 * 1024;
const DEFAULT_MAX_ITER = 256;

class EngineTileSource extends OpenSeadragon.TileSource {
  private parent: Viewer;
  private algorithm: string;
  private max_iter: number;
  private metadata: FractalView = new FractalView();
  private colourer: string;
  // We don't have the metadata at ctor time, so this lets us async on it:
  private metadata_promise_: PromiseLike<FractalView>;

  // CAUTION: Immediately after construction, metadata is not valid until after it has round-tripped to the engine to get the metadata.
  // (Could add a validity flag or something eventish if needed.)
  constructor(parent: Viewer, algorithm: string, max_iter: number, colourer: string) {
    super({
      height: IMAGE_DIMENSION,
      width: IMAGE_DIMENSION,
      tileSize: TILE_SIZE,
      minLevel: 8,
      tileOverlap: 0,
    });
    this.parent = parent;
    this.algorithm = algorithm;
    this.max_iter = max_iter;
    this.colourer = colourer;
    let metadata_resolve: any = null;
    this.metadata_promise_ = new Promise((resolve) => { metadata_resolve = resolve; });
    invoke('get_metadata', { algorithm: algorithm })
      .then((reply) => {
        let meta = FractalView.fromDict(reply);
        this.metadata.axes_length = meta.axes_length;
        this.metadata.origin = meta.origin;
        metadata_resolve(this.metadata);
      })
      .catch((e) => {
        console.log(`Error retrieving metadata for ${algorithm}: ${e}`);
      });
  }

  get_algorithm(): string { return this.algorithm; }
  get_max_iter(): number { return this.max_iter; }
  get_metadata(): FractalView { return this.metadata; }
  get_colourer(): string { return this.colourer; }
  metadata_promise(): PromiseLike<FractalView> { return this.metadata_promise_; }

  getTileUrl(level: number, x: number, y: number): string {
    return `${this.algorithm}:${this.max_iter}/${this.colourer}/${level}/${x}-${y}}`;
  }

  // caution: @types/openseadragon 3.0.10 doesn't know about these functions
  getTilePostData(level: number, x: number, y: number) {
    return new TilePostData(level, x, y);
  }

  downloadTileStart(context: any /* OpenSeadragon.ImageJob */) {
    // tile dx and dy are the column and row numbers FOR THE ZOOM LEVEL.
    // Given 1048576x1048576 pixels, we start at level 10 (4x4 tiles comprise the image) and end at level 20 (4096x4096)
    // => At zoom level X, the image is 2^X pixels across.

    let spec = new TileSpec(nextSerial(), context?.postData, TILE_SIZE, TILE_SIZE, this.algorithm, this.max_iter, this.colourer);
    context.userData = spec;
    this.parent.add_outstanding_request(spec.serial, context);
    invoke('start_tile', {
      spec: spec
    })
      .catch((e) => {
        context?.finish?.(null, null, e.toString());
      });
  }

  downloadTileAbort(context: any /*OpenSeadragon.ImageJob*/) {
    console.log(`OSD requested tile abort: tile #${context.userData.serial}`);
    invoke('abort_tile', { serial: context.userData.serial })
      .catch((e) => {
        context.finish?.(null, null, e.toString());
      });
  }
  createTileCache(cache: any/*CacheObject*/, data: any) {
    cache._data = data;
  }
  destroyTileCache(cache: any/*CacheObject*/) {
    cache._data = null;
  }
  getTileCacheData(cache: any/*CacheObject*/) {
    return cache._data;
  }
  getTileCacheDataAsImage() {
    // not implementing all the features brings limitations to the
    // system, namely tile.getImage() will not work and also
    // html-based drawing approach will not work
    throw "getTileCacheDataAsImage not implemented";
  }

  getTileCacheDataAsContext2D(cache: any/*CacheObject*/) {
    // our data is already context2D - what a luck!
    return cache._data;
  }

}

export class Viewer {
  private osd: any | undefined;  // OpenSeadragon.Viewer
  private redraw_event: number | undefined; // setTimeout / clearTimeout
  private unlisten_tile_complete: UnlistenFn | null = null;
  private unlisten_tile_error: UnlistenFn | null = null;
  private outstanding_requests: Map<number, any/*OpenSeadragon.ImageJob*/> = new Map();
  private hud_: HeadsUpDisplay;
  private all_fractals: ListItem[] = [];
  private all_colourers: ListItem[] = [];
  private isFullyLoaded: boolean = false;

  // width, height used by coordinate display
  private width_: number = NaN;
  private height_: number = NaN;
  width(): number { return this.width_; }
  height(): number { return this.height_; }
  hud(): HeadsUpDisplay { return this.hud_; }

  private busyIndicator: HTMLElement;

  constructor() {
    let self = this; // Closure helper

    invoke('list_items', { what: 'fractals' })
      .then((reply) => {
        self.all_fractals = reply as ListItem[];
        return invoke('list_items', { what: 'colourers' });
      })
      .then((reply) => {
        self.all_colourers = reply as ListItem[];

        let initialSource = new EngineTileSource(this, self.all_fractals[0].name, DEFAULT_MAX_ITER, self.all_colourers[0].name);
        this.osd = OpenSeadragon({
          id: "openseadragon",
          prefixUrl: "/openseadragon/images/",
          homeFillsViewer: true,
          autoResize: true,
          preserveImageSizeOnResize: true,
          visibilityRatio: 1.0,
          debugMode: false,
          showRotationControl: false,
          showNavigator: true,
          navigatorAutoFade: true,
          navigatorBackground: '#FFF',
          showFullPageControl: false,
          zoomPerSecond: 2.0,
          toolbar: "topbar",
          constrainDuringPan: true,
          navigatorDisplayRegionColor: '#789',
          placeholderFillStyle: '#789',

          tileSources: [initialSource],
        }); // ---------------- end this.osd initialiser ---------------------------

        this.bind_events().then(() => {
          this.osd.addHandler("before-destroy", function () {
            self.unlisten_tile_complete?.();
            self.unlisten_tile_error?.();
          });
        });

        // Window resize
        // Rather than caning the system as we get a resize event for every pixel, add a slight debounce.
        // Note we call out to the Viewer class ('self') to resize.
        window.addEventListener('resize', function (_event) {
          if (self.redraw_event !== undefined) {
            this.clearTimeout(self.redraw_event);
          }
          self.redraw_event = this.setTimeout(function () {
            self.resize();
            self.redraw_event = undefined;
          }, 100);
        }, true);

        let viewer = this.osd;
        viewer.addOnceHandler('open', function () {
          viewer.addHandler('animation', () => { self.updateIndicator() });
          window.setTimeout(() => {
            // running this immediately on open is too early; axes_length is zero, which breaks the hud
            self.updateIndicator();
          }, 50);
        });

        viewer.world.addHandler('add-item', function (event: any) {
          var tiledImage = event.item;
          tiledImage.addHandler('fully-loaded-change', function () {
            var newFullyLoaded = self.areAllFullyLoaded();
            if (newFullyLoaded !== self.isFullyLoaded) {
              self.isFullyLoaded = newFullyLoaded;
              self.updateBusyIndicator();
            }
          });
        });

        this.resize(); // also computes home margin, but we don't reliably have the metadata in yet
        initialSource.metadata_promise()
          .then((_) => {
            self.compute_home_margins(initialSource);
            self.osd.viewport.goHome();
          });
      }); // ---- end long 'then' block ----

    // Zoom/Position indicator
    this.hud_ = new HeadsUpDisplay(document);
    this.busyIndicator = (document.querySelector('.busy')) as HTMLElement;
  } // ---------------- end constructor --------------------

  private metadata(): FractalView {
    return this.get_active_source().get_metadata();
  }

  private updateBusyIndicator() {
    if (this.busyIndicator === null) return;
    if (this.isFullyLoaded) {
      this.busyIndicator.style.display = 'none';
    } else {
      this.busyIndicator.style.display = 'block';
    }
  }

  private forceBusyIndicator() {
    this.isFullyLoaded = false;
    this.updateBusyIndicator();
  }

  private areAllFullyLoaded() {
    var tiledImage;
    var count = this.osd.world.getItemCount();
    for (var i = 0; i < count; i++) {
      tiledImage = this.osd.world.getItemAt(i);
      if (!tiledImage.getFullyLoaded()) {
        return false;
      }
    }
    return true;
  }

  updateIndicator() {
    if (this.osd === undefined || this.hud_ === undefined) {
      return;
    }
    try {
      let vp = this.osd.viewport;
      var zoom: number = vp.getZoom(true);
      let position = this.get_position();
      this.hud_.update(zoom, position.origin, position.centre(), position.axes_length, this.width_, this.height_, this.get_algorithm(), this.get_colourer());
    }
    catch (e) {
      //let err = e as Error;
      console.error(`exception in UpdateIndicator: ${e}`); // this includes the stack trace
    }
  }

  get_position(): FractalView {
    let viewer = this.osd;
    let vp = viewer.viewport;
    // We know that top left is webPoint 0,0; bottom right is W-1,H-1.
    // These are the web (pixel) coordinates.
    var topLeft = new OpenSeadragon.Point(0, 0);
    var bottomRight = new OpenSeadragon.Point(this.width_ - 1, this.height_ - 1);
    // Convert to viewport coordinates:
    var topLeftView = vp.pointFromPixelNoRotate(topLeft);
    var bottomRightView = vp.pointFromPixelNoRotate(bottomRight);

    // Bottom Left is the origin (as mathematicians would call it, not computer images!)
    var originView = new OpenSeadragon.Point(topLeftView.x, bottomRightView.y);

    // Axes := BR - TL
    var axesLengthView = bottomRightView.minus(topLeftView);

    // Convert to complex
    let meta = this.metadata();
    let meta_axes = meta.axes_length;

    var originComplex = new EnginePoint(
      meta.origin.re + originView.x * meta_axes.re,
      // Flip the Y axis at the point we go into mathematician-speak:
      meta.origin.im + (1.0 - originView.y) * meta_axes.im
    );
    var axesComplex = new EnginePoint(
      axesLengthView.x * meta_axes.re,
      axesLengthView.y * meta_axes.im
    );

    return new FractalView(originComplex, axesComplex);
  }

  async bind_events() {
    this.unlisten_tile_complete = await listen<TileResponse>('tile_complete', (event) => {
      this.on_tile_complete(event.payload);
    });
    this.unlisten_tile_error = await listen<TileError>('tile_error', (event) => {
      this.on_tile_error(event.payload);
    });
    // Note the before-destroy handler we set up elsewhere.
  }

  add_outstanding_request(key: number, value: any) {
    this.outstanding_requests.set(key, value);
  }


  on_tile_complete(response: TileResponse) {
    let context = this.outstanding_requests.get(response.serial);
    if (context === undefined) return; // It's not for us
    //let spec:TileSpec = context.userData;
    //console.log(`got tile #${response.serial} = ${spec.level}/${spec.dx}-${spec.dy}`);
    this.outstanding_requests.delete(response.serial);

    // "convert the data to a canvas and return its 2D context"
    let response2 = new TileResponseHelper(response);
    let image = response2.image(TILE_SIZE);
    let canvas = document.createElement("canvas");
    canvas.width = TILE_SIZE;
    canvas.height = TILE_SIZE;
    let ctx2d = canvas.getContext("2d");
    ctx2d?.putImageData(image, 0, 0);
    context.finish(ctx2d);
  }
  on_tile_error(err: TileError) {
    let context = this.outstanding_requests.get(err.serial);
    //let spec:TileSpec = context.userData;
    context?.finish?.(null, null, err.error);
  }

  resize() {
    // Dynamically size to fill the window
    let viewerElement = jQuery('#openseadragon');
    viewerElement.height(window.innerHeight);
    viewerElement.width(window.innerWidth);
    this.width_ = window.innerWidth;
    this.height_ = window.innerHeight;
    this.osd.viewport.resize({ x: window.innerWidth, y: window.innerHeight });
    this.osd.viewport.applyConstraints();
    console.log(`Window resized to ${window.innerWidth} x ${window.innerHeight}`);
    // We must update the margins (this may not succeed on startup, if the metadata isn't loaded yet)
    this.compute_home_margins(this.osd.tileSources[this.osd.currentPage()]);
  }

  go_to_position(destination: UserDestination) {
    let messageBox = document.getElementById("position-error-text");
    try {
      let result = this.go_to_inner(destination);
      messageBox!.innerHTML = "";
      return result;
    } catch (e) {
      console.error(e);
      messageBox!.innerHTML = (e as Error)!.toString();
    }
  }
  private go_to_inner(destination: UserDestination) {
    let viewport = this.osd.viewport;
    // this is essentially the inverse of updateIndicator()

    let meta = this.metadata();
    let meta_axes = meta.axes_length;

    let originComplex = undefined;
    let centreComplex = undefined;
    if (this.hud_.origin_is_currently_visible()) {
      originComplex = new EnginePoint(destination.originReal, destination.originImag);
      if (!Number.isFinite(originComplex.re) || !Number.isFinite(originComplex.im)) {
        throw new Error("Origin is required");
      }
      console.log("Go to origin:", originComplex);
    } else {
      centreComplex = new EnginePoint(destination.centreReal, destination.centreImag);
      if (!Number.isFinite(centreComplex.re) || !Number.isFinite(centreComplex.im)) {
        throw new Error("Centre is required");
      }
      console.log("Go to centre:", centreComplex);
    }


    // Which axis-controlling coordinates are we using?
    // The first one (left to right) takes precedence.
    let axesReal = destination.axesReal;
    let axesImag = destination.axesImag;
    let zoom = destination.zoom;
    // Assume square pixels.
    let aspectRatio = this.width_ / this.height_;
    if (Number.isFinite(axesReal)) {
      axesImag = axesReal / aspectRatio;
      zoom = meta.axes_length.re / axesReal;
    } else if (Number.isFinite(axesImag)) {
      axesReal = axesImag * aspectRatio;
      zoom = meta.axes_length.re / axesReal;
    } else if (Number.isFinite(zoom)) {
      axesReal = meta.axes_length.re / zoom;
      axesImag = axesReal / aspectRatio;
    } else {
      throw new Error("Axis length must be specified");
    }
    console.log("destination axis", new EnginePoint(axesReal, axesImag));
    console.log("destination zoom", zoom);

    // 1. Compute axes in viewport coordinates (0..1)
    let axesLengthView = new OpenSeadragon.Point(
      axesReal / meta_axes.re,
      axesImag / meta_axes.im,
    );

    // 2. Convert centre to origin (both complex)
    if (originComplex === undefined) {
      if (centreComplex === undefined) {
        throw new Error("Need centre or origin");
      }
      originComplex = new EnginePoint(
        centreComplex.re - 0.5 * axesReal,
        centreComplex.im - 0.5 * axesImag,
      );
    }

    // 3. Compute origin in viewport coordinates
    // (this is a mathematician's origin i.e. bottom left)
    let originView = new OpenSeadragon.Point(
      (originComplex.re - meta.origin.re) / meta_axes.re,
      // flip the Y axis as we're going between maths and computer science coordinates here
      1.0 - (originComplex.im - meta.origin.im) / meta_axes.im,
    );

    // 4. Use origin point & axes length to compute top-left & bottom-right points, all in viewport coordinates.
    // AxesLength = BR - TL
    let topLeftView = new OpenSeadragon.Point(originView.x, originView.y - axesLengthView.y);
    let bottomRightView = topLeftView.plus(axesLengthView);
    let centreView = new OpenSeadragon.Point(
      (topLeftView.x + bottomRightView.x) / 2.0,
      (topLeftView.y + bottomRightView.y) / 2.0,
    );

    console.log("Destination centre", centreView);
    viewport.zoomTo(zoom).panTo(centreView);
    viewport.applyConstraints();
  }

  // Copy the current position into the Go To Position form
  copy_current_position() {
    let pos = this.get_position();
    this.hud_.set_go_to_position(pos, this.width_, this.height_);
  }

  get_active_source(): EngineTileSource {
    let index = this.osd.currentPage();
    return this.osd.world?.getItemAt(index)?.source as EngineTileSource;
  }

  get_max_iter() {
    return this.get_active_source().get_max_iter();
  }
  set_max_iter(new_max: number) {
    if (Number.isFinite(new_max)) {
      let oldSource = this.get_active_source();
      let newSource = new EngineTileSource(this, oldSource.get_algorithm(), new_max, oldSource.get_colourer());
      this.change_source_keeping_position(newSource);
    } else {
      console.warn(`failed to parse max_iter ${new_max}`);
    }
  }
  change_source_keeping_position(newSource: EngineTileSource) {
    // Stash the position:
    let viewport = this.osd!.viewport;
    let centre = viewport.getCenter();
    let zoom = viewport.getZoom();

    this.replace_active_source(newSource);
    // This causes a new viewport to be opened for the new source.
    let self = this; // for closure
    this.osd.addOnceHandler('open', function () {
      let viewport = self.osd!.viewport; // because you get a new viewport on the new source
      viewport.zoomTo(zoom, null, true).panTo(centre, true);
      // Curveball: Sometimes (sadly quite often) the fractal metadata are not ready at this point,
      // naively calling updateIndicator() here fails. We must async it:
      self.get_active_source().metadata_promise()
        .then(() => {
          self.updateIndicator();
          self.forceBusyIndicator();
        });
    });
  }
  get_algorithm(): string {
    return this.get_active_source().get_algorithm();
  }
  get_colourer(): string {
    return this.get_active_source()?.get_colourer();
  }
  set_algorithm(new_fractal: string) {
    let oldSource = this.get_active_source();
    let newSource = new EngineTileSource(this, new_fractal, oldSource.get_max_iter(), oldSource.get_colourer());
    this.replace_active_source(newSource);
    this.osd.viewport.goHome();
    let self = this;
    this.osd.addOnceHandler('open', function () {
      self.get_active_source().metadata_promise()
        .then(() => {
          self.updateIndicator();
          self.forceBusyIndicator();
        });
    });
  }
  set_colourer(new_colourer: string) {
    let oldSource = this.get_active_source();
    if (oldSource === undefined) {
      // If the user leans on one of the keyboard shortcuts to cycle colourer, sometimes OSD is mid-change when the next event comes through.
      return;
    }
    let newSource = new EngineTileSource(this, oldSource.get_algorithm(), oldSource.get_max_iter(), new_colourer);
    // This is a recolour event, so we do NOT want to go home (the default when changing source).
    this.change_source_keeping_position(newSource);
  }
  cycle_colourer(delta: number) {
    let needle = this.get_colourer();
    if (needle === null || needle === undefined) {
      // If the user leans on one of the keyboard shortcuts to cycle colourer, sometimes OSD is mid-change when the next event comes through.
      return;
    }
    let index = this.all_colourers.findIndex((value: ListItem): boolean => { return value.name == needle });
    if (index == -1) {
      console.warn(`Couldn't determine index of current colourer ${needle}`);
      console.log(this.all_colourers);
      index = 0;
    } else {
      // Add delta, mod size of list
      index = index + delta;
      if (index < 0) index = this.all_colourers.length - 1;
      else if (index >= this.all_colourers.length) index = 0;
    }
    this.set_colourer(this.all_colourers[index].name);
  }

  // Something important changed (algorithm, max_iter, etc). Replace the active source.
  private replace_active_source(source: EngineTileSource) {
    // This causes a canvas flash.
    // There is a workaround described in https://github.com/openseadragon/openseadragon/issues/1991 which we used to have (as redraw()) but that interferes with multi-image mode, so leave off for now.
    this.osd._cancelPendingImages();
    this.osd.open(source);
    // new source, might need different margins
    this.compute_home_margins(source);
    // but do NOT goHome here, that may not be desirable (e.g. changing colourer)
  }

  private nav_visible: boolean = true;
  toggle_navigator() {
    let element = this.osd.navigator.element;
    if (this.nav_visible)
      element.style.display = "none";
    else
      element.style.display = "inline-block";
    this.nav_visible = !this.nav_visible;
  }

  private compute_home_margins(src: EngineTileSource) {
    // The source or window size has changed.
    // What margins are needed to have the whole fractal on screen in the home position?
    let meta = src.get_metadata();
    let fractal_aspect = meta.axes_length.re / meta.axes_length.im;
    let screen_aspect = this.width_ / this.height_;
    let margins = { left: 0, top: 0, right: 0, bottom: 0 };
    if (!isFinite(fractal_aspect) || !isFinite(screen_aspect)) {
      // This happens before the fractal metadata have loaded. Ignore.
      return;
    }
    let diff = screen_aspect - fractal_aspect;
    if (diff > 0.0) {
      // Screen is wider than fractal.
      // The difference tells us how many multiples of the fractal width should form the margin.
      let each_side = diff / 2.0;
      // At the home position the fractal height fills the viewport vertically. Therefore:
      let each_side_pixels = each_side * this.height_;
      margins.left = each_side_pixels;
      margins.right = each_side_pixels;
    } else if (diff < 0.0) {
      // Screen is narrower than fractal.
      // Compute how many multiples of the fractal height should form the margin.
      let vertical_margin = (1.0 / screen_aspect) - (1.0 / fractal_aspect);
      let each_margin = vertical_margin / 2.0;
      // At the home position the fractal width fills the viewport horizontally. Therefore:
      let margin_pixels = each_margin * this.width_;
      margins.top = margin_pixels;
      margins.bottom = margin_pixels;
    } else {
      // no margins
    }
    this.osd.viewport.setMargins(margins);
  }
}
