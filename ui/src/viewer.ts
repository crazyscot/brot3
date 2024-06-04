// brot3 fractal viewer (bridge to OpenSeadragon)
// (c) 2024 Ross Younger

import { invoke } from '@tauri-apps/api'
import { UnlistenFn, listen } from '@tauri-apps/api/event'
import jQuery from 'jquery'
import OpenSeadragon from 'openseadragon'

import { EnginePoint, FractalView, TileSpec, TileResponse, TileError, TilePostData, TileResponseHelper } from './engine_types'
import { HeadsUpDisplay, UserDestination } from './hud'
import { nextSerial } from './serial_allocator'

export const TILE_SIZE = 128;
const IMAGE_DIMENSION = 1024 * 1024 * 1024 * 1024;
const DEFAULT_ALGORITHM = "Original";
const DEFAULT_MAX_ITER = 256;
const DEFAULT_COLOURER = "LinearRainbow";

class EngineTileSource extends OpenSeadragon.TileSource {
  private parent: Viewer;
  private algorithm: string;
  private max_iter: number;
  private metadata: FractalView = new FractalView();
  private colourer: string;

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
    invoke('get_metadata', { algorithm: algorithm })
      .then((reply) => {
        let meta = reply as FractalView;
        this.metadata.axes_length = meta.axes_length;
        this.metadata.origin = meta.origin;
      })
      .catch((e) => {
        console.log(`Error retrieving metadata for ${algorithm}: ${e}`);
      });
  }

  get_algorithm(): string { return this.algorithm; }
  get_max_iter(): number { return this.max_iter; }
  get_metadata(): FractalView { return this.metadata; }
  get_colourer(): string { return this.colourer; }

  getTileUrl(level: number, x: number, y: number): string {
    // TODO add colour (or we'll break cacheing!)
    return `${this.algorithm}:${this.max_iter}/${level}/${x}-${y}}`;
  }

  // caution: @types/openseadragon 3.0.10 doesn't know about these functions
  getTilePostData(level: number, x: number, y: number) {
    return new TilePostData(level, x, y);
  }

  async downloadTileStart(context: any /* OpenSeadragon.ImageJob */) {
    // tile dx and dy are the column and row numbers FOR THE ZOOM LEVEL.
    // Given 1048576x1048576 pixels, we start at level 10 (4x4 tiles comprise the image) and end at level 20 (4096x4096)
    // => At zoom level X, the image is 2^X pixels across.

    let spec = new TileSpec(await nextSerial(), context?.postData, TILE_SIZE, TILE_SIZE, this.algorithm, this.max_iter, this.colourer);
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
  private osd: any | null;  // OpenSeadragon.Viewer
  private redraw_event: number | undefined; // setTimeout / clearTimeout
  private unlisten_tile_complete: UnlistenFn | null = null;
  private unlisten_tile_error: UnlistenFn | null = null;
  private outstanding_requests: Map<number, any/*OpenSeadragon.ImageJob*/> = new Map();
  private hud_: HeadsUpDisplay;

  // width, height used by coordinate display
  private width_: number = NaN;
  private height_: number = NaN;
  width(): number { return this.width_; }
  height(): number { return this.height_; }
  hud(): HeadsUpDisplay { return this.hud_; }

  constructor() {
    let self = this; // Closure helper

    let initialSource = new EngineTileSource(this, DEFAULT_ALGORITHM, DEFAULT_MAX_ITER, DEFAULT_COLOURER);

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
      showFullPageControl: false,
      zoomPerSecond: 2.0,
      toolbar: "topbar",

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
      }, 100);
    }, true);

    // Zoom/Position indicator
    this.hud_ = new HeadsUpDisplay(document);
    let viewer = this.osd;
    viewer.addHandler('open', function () {
      viewer.addHandler('animation', () => { self.updateIndicator() });
    });

    // Initial position at constructor time is not correct, so defer it; only a tiny deferral seems needed
    // TODO figure out why this is and make it suitably event-based; could be waiting on OSD ?
    window.setTimeout(function () { self.updateIndicator(); }, 10);

  } // ---------------- end constructor --------------------

  private metadata(): FractalView {
    return this.get_active_source().get_metadata();
  }

  updateIndicator() {
    if (this.osd === undefined || this.hud_ === undefined) {
      return;
    }
    let vp = this.osd!.viewport;
    var zoom: number = vp!.getZoom(true);
    let position = this.get_position();
    this.hud_!.update(zoom, position!.origin, position!.centre(), position!.axes_length, this.width_, this.height_);
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
    return this.osd.world.getItemAt(index).source as EngineTileSource;
  }

  get_max_iter() {
    return this.get_active_source().get_max_iter();
  }
  set_max_iter(new_max: number) {
    if (Number.isFinite(new_max)) {
      let oldSource = this.get_active_source();
      let newSource = new EngineTileSource(this, oldSource.get_algorithm(), new_max, oldSource.get_colourer());
      this.replace_active_source(newSource);
    } else {
      console.warn(`failed to parse max_iter ${new_max}`);
    }
  }
  get_algorithm(): string {
    return this.get_active_source().get_algorithm();
  }
  get_colourer(): string {
    return this.get_active_source().get_colourer();
  }
  set_algorithm(new_fractal: string) {
    let oldSource = this.get_active_source();
    let newSource = new EngineTileSource(this, new_fractal, oldSource.get_max_iter(), oldSource.get_colourer());
    this.replace_active_source(newSource);
    this.osd.viewport.goHome();
    window.setTimeout(() => { this.updateIndicator(); }, 10);
  }

  // Something important changed (algorithm, max_iter, etc). Replace the active source.
  private replace_active_source(source: EngineTileSource) {
    // This causes a canvas flash.
    // There is a workaround described in https://github.com/openseadragon/openseadragon/issues/1991 which we used to have (as redraw()) but that interferes with multi-image mode, so leave off for now.
    this.osd._cancelPendingImages();
    this.osd.open(source);
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

  // dummy function to shut up a linter warning in main.ts
  noop() { }
}
