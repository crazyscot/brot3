// brot3 fractal viewer (bridge to OpenSeadragon)
// (c) 2024 Ross Younger

import { invoke } from '@tauri-apps/api'
import { UnlistenFn, listen } from '@tauri-apps/api/event'
import jQuery from 'jquery'
import OpenSeadragon from 'openseadragon'

import { SerialAllocator } from './serial_allocator'
import { EnginePoint, FractalMetadata, TileSpec, TileResponse, TileError, TilePostData } from './engine_types'

var gSerial = new SerialAllocator();
const TILE_SIZE = 128;
const IMAGE_DIMENSION = 1024 * 1024 * 1024 * 1024;

function maybe_leading(symbol: string, n: number) : string
{
  if (n >= 0.0)
    return `${symbol}${n}`;
  return `${n}`;
}

function tr_is_visible(e: HTMLElement): boolean {
  return e.style.display !== "none";
}

// Toggles visibility of a TR element, returning true iff it is now visible.
function toggle_tr_visibility(e: HTMLElement) : boolean {
  if (e.style.display === "none") {
      e.style.display = "table-row";
      return true;
  } else {
      e.style.display = "none";
      return false;
  }
}

class HeadsUpDisplay {
  zoom: Element | null;
  originReal: Element | null;
  originImag: Element | null;
  centreReal: Element | null;
  centreImag: Element | null;
  axesReal: Element | null;
  axesImag: Element | null;
  constructor(doc: Document) {
    var panel = doc.querySelectorAll('#info-panel')[0];
    this.zoom = panel.querySelectorAll('#zoom')[0];
    this.originReal = panel.querySelectorAll('#originReal')[0];
    this.originImag = panel.querySelectorAll('#originImag')[0];
    this.centreReal = panel.querySelectorAll('#centreReal')[0];
    this.centreImag = panel.querySelectorAll('#centreImag')[0];
    this.axesReal = panel.querySelectorAll('#axesReal')[0];
    this.axesImag = panel.querySelectorAll('#axesImag')[0];
  }
  update(zoom: number, origin: EnginePoint, centre: EnginePoint, axes: EnginePoint) {
    this.zoom!.innerHTML = `${zoom.toPrecision(4)} &times;`;
    this.axesReal!.innerHTML = maybe_leading("&nbsp;", axes.re);
    this.axesImag!.innerHTML = maybe_leading("&nbsp;", axes.im);
    this.centreReal!.innerHTML = maybe_leading("&nbsp;", centre.re);
    this.centreImag!.innerHTML = maybe_leading("+", centre.im);
    this.originReal!.innerHTML = maybe_leading("&nbsp;", origin.re);
    this.originImag!.innerHTML = maybe_leading("+", origin.im);
  }
}

class ClickEventListener implements EventListenerObject {
  clickable: Element;
  action: Function;
  constructor(target: Element, action: Function) {
    this.clickable = target;
    this.clickable.addEventListener("click", this);
    this.action = action;
  }
  handleEvent(event: Event): void | Promise<void> {
    if (event.type === "click") {
        this.action(event);
    }
  }
}

export class Viewer {
  osd: any | null;  // OpenSeadragon.Viewer
  redraw_event: number | undefined; // setTimeout / clearTimeout
  unlisten_tile_complete: UnlistenFn | null = null;
  unlisten_tile_error: UnlistenFn | null = null;
  outstanding_requests: Map<number, any/*OpenSeadragon.ImageJob*/> = new Map();
  hud: HeadsUpDisplay;
  current_metadata: FractalMetadata = new FractalMetadata();
  hud_closer: ClickEventListener;
  position_entry_closer: ClickEventListener;

  // width, height used by coordinate display
  width: number = NaN;
  height: number = NaN;

  constructor() {
    let self = this; // Closure helper

    this.osd = OpenSeadragon({
      id:         "openseadragon",
      prefixUrl: "/openseadragon/images/",
      homeFillsViewer: true,
      autoResize: true,
      preserveImageSizeOnResize: true,
      visibilityRatio: 1.0,
      debugMode: false,
      showRotationControl: false,
      showNavigator: false,
      showFullPageControl: false,
      zoomPerSecond: 2.0,
      toolbar: "topbar",

      tileSources: {
        height: IMAGE_DIMENSION,
        width: IMAGE_DIMENSION,
        tileSize: TILE_SIZE,
        minLevel: 8,
        tileOverlap: 0,

        getTileUrl: function (level: number, x: number, y: number) {
          // TODO add fractal, colour (or we'll break cacheing!)
          return `${level}/${x}-${y}`;
        },
        // caution: @types/openseadragon 3.0.10 doesn't know about these functions
        getTilePostData: function (level: number, x: number, y: number) {
          // TODO add fractal, colour
          return new TilePostData(level, x, y);
        },
        downloadTileStart: async function (context: any /* OpenSeadragon.ImageJob */) {
          // tile dx and dy are the column and row numbers FOR THE ZOOM LEVEL.
          // Given 1048576x1048576 pixels, we start at level 10 (4x4 tiles comprise the image) and end at level 20 (4096x4096)
          // => At zoom level X, the image is 2^X pixels across.

          let spec = new TileSpec(await gSerial.next(), context?.postData, TILE_SIZE, TILE_SIZE);
          context.userData = spec;
          self.outstanding_requests.set(spec.serial, context);
          invoke('start_tile', {
            spec: spec
          })
          .catch((e) => {
            context?.finish?.(null, null, e.toString());
          });
        },
        downloadTileAbort: function (context: any /*OpenSeadragon.ImageJob*/) {
          console.log(`OSD requested tile abort: tile #${context.userData.serial}`);
          invoke('abort_tile', { serial: context.userData.serial })
          .catch((e) => {
            context.finish?.(null, null, e.toString());
          });
        },
        createTileCache: function (cache: any/*CacheObject*/, data: any) {
          cache._data = data;
        },
        destroyTileCache: function (cache: any/*CacheObject*/) {
          cache._data = null;
        },
        getTileCacheData: function(cache: any/*CacheObject*/) {
          return cache._data;
        },
        getTileCacheDataAsImage: function() {
          // not implementing all the features brings limitations to the
          // system, namely tile.getImage() will not work and also
          // html-based drawing approach will not work
          throw "getTileCacheDataAsImage not implemented";
        },

        getTileCacheDataAsContext2D: function(cache: any/*CacheObject*/) {
          // our data is already context2D - what a luck!
          return cache._data;
        },
      },
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
    this.hud = new HeadsUpDisplay(document);
    let viewer = this.osd;
    var updateIndicator = function () {
      let vp = viewer.viewport;
      var zoom: number = vp.getZoom(true);
      let position = self.get_position();
      self.hud.update(zoom, position.origin, position.centre(), position.axes_length);
      /*
      let checkZoom = self.current_metadata.axes_length.re / axesComplex.re;
      console.log(`real: meta ${self.current_metadata.axes_length.re}, axis ${axesComplex.re}, zoom ${zoom}, computed zoom = ${checkZoom}`);
      */
    }
    viewer.addHandler('open', function () {
      viewer.addHandler('animation', updateIndicator);
    });
    // Retrieve initial metadata.
    invoke('get_metadata')
      .then((reply) => {
        // TODO when we have selectable fractals, this will need to be updated.
        // Careful, current_metadata is captured by a closure.
        let meta = reply as FractalMetadata;
        this.current_metadata.axes_length = meta.axes_length;
        this.current_metadata.origin = meta.origin;
        // Initial position at constructor time is not correct, so defer it; only a tiny deferral seems needed
        // TODO figure out why this is and make it suitably event-based; could be waiting on OSD ?
        window.setTimeout(function () { updateIndicator(); }, 10);
      })
      .catch((e) => {
        console.log(`Error retrieving metadata: ${e}`);
      }
    );

    // Hide the position entry rows by default
    this.position_entry_rows().forEach(e => { if (tr_is_visible(e)) toggle_tr_visibility(e); });
    this.hud_closer = new ClickEventListener(
      document.getElementById("close-hud")!,
      function (_event: Event) {
        self.toggle_hud();
      }
    );
    this.position_entry_closer = new ClickEventListener(
      document.getElementById("close-entry")!,
      function (_event: Event) {
        self.toggle_position_entry_panel();
      }
    );
  } // ---------------- end constructor --------------------

  get_position() : FractalMetadata {
    let viewer = this.osd;
    let vp = viewer.viewport;
    // We know that top left is webPoint 0,0; bottom right is W-1,H-1.
    // These are the web (pixel) coordinates.
    var topLeft = new OpenSeadragon.Point(0, 0);
    var bottomRight = new OpenSeadragon.Point(this.width - 1, this.height - 1);
    // Convert to viewport coordinates:
    var topLeftView = vp.pointFromPixelNoRotate(topLeft);
    var bottomRightView = vp.pointFromPixelNoRotate(bottomRight);

    // Bottom Left is the origin (as mathematicians would call it, not computer images!)
    var originView = new OpenSeadragon.Point(topLeftView.x, bottomRightView.y);

    // Axes := BR - TL
    var axesLengthView = bottomRightView.minus(topLeftView);

    // Convert to complex
    let meta = this.current_metadata; // Caution, closure capture!
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

    return new FractalMetadata(originComplex, axesComplex);
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

  on_tile_complete(response: TileResponse) {
    let context = this.outstanding_requests.get(response.serial);
    //let spec:TileSpec = context.userData;
    //console.log(`got tile #${response.serial} = ${spec.level}/${spec.dx}-${spec.dy}`);

    // "convert the data to a canvas and return its 2D context"
    // response.rgba_blob is a byte array
    let blob = new Uint8ClampedArray(response.rgba_blob);
    let image = new ImageData(blob, TILE_SIZE, TILE_SIZE, { "colorSpace": "srgb" });
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
    this.width = window.innerWidth;
    this.height = window.innerHeight;
    this.osd.viewport.resize({x: window.innerWidth, y:window.innerHeight});
    this.osd.viewport.applyConstraints();
    console.log(`Window resized to ${window.innerWidth} x ${window.innerHeight}`);
  }

  toggle_hud() {
    let elements = Array.from(document.querySelectorAll('tr.position-display'), e => e as HTMLElement);
    elements.forEach(e => toggle_tr_visibility(e));
  }

  private position_entry_rows() : HTMLElement[] {
    return Array.from(document.querySelectorAll('tr.position-entry'), e => e as HTMLElement);
  }

  toggle_position_entry_panel() {
    let visible = false;
    this.position_entry_rows().forEach(e => visible = toggle_tr_visibility(e));
    if (visible) {
      let element = undefined;
      if (this.origin_is_currently_visible()) {
        element = document.getElementById(`enter_originReal`) as HTMLInputElement;
      } else {
        element = document.getElementById(`enter_centreReal`) as HTMLInputElement;
      }
      element!.focus();
      element!.select();
    }
  }

  go_to(destination: Map<string, number>) {
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
  private go_to_inner(destination: Map<string, number>) {
    let viewport = this.osd.viewport;
    // this is essentially the inverse of updateIndicator()

    let meta = this.current_metadata;
    let meta_axes = meta.axes_length;

    let originComplex = undefined;
    let centreComplex = undefined;
    if (this.origin_is_currently_visible()) {
      originComplex = new EnginePoint(destination.get("originReal")!, destination.get("originImag")!);
      if (!Number.isFinite(originComplex.re) || !Number.isFinite(originComplex.im)) {
        throw new Error("Origin is required");
      }
      console.log("Go to origin:", originComplex);
    } else {
      centreComplex = new EnginePoint(destination.get("centreReal")!, destination.get("centreImag")!);
      if (!Number.isFinite(centreComplex.re) || !Number.isFinite(centreComplex.im)) {
        throw new Error("Centre is required");
      }
      console.log("Go to centre:", centreComplex);
    }


    // Which axis-controlling coordinates are we using?
    // The first one (left to right) takes precedence.
    let axesReal = destination.get("axesReal")!;
    let axesImag = destination.get("axesImag")!;
    let zoom = destination.get("zoom")!;
    // Assume square pixels.
    let aspectRatio = this.width / this.height;
    if (Number.isFinite(axesReal)) {
      axesImag = axesReal / aspectRatio;
      zoom = this.current_metadata.axes_length.re / axesReal;
    } else if (Number.isFinite(axesImag)) {
      axesReal = axesImag * aspectRatio;
      zoom = this.current_metadata.axes_length.re / axesReal;
    } else if (Number.isFinite(zoom)) {
      axesReal = this.current_metadata.axes_length.re / zoom;
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
      ( originComplex.re - meta.origin.re ) / meta_axes.re,
      // flip the Y axis as we're going between maths and computer science coordinates here
      1.0 - ( originComplex.im - meta.origin.im ) / meta_axes.im,
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

    let f = document.getElementById("enter_axesReal")! as HTMLInputElement;
    f.value = pos.axes_length.re.toString();

    if (this.origin_is_currently_visible()) {
      f = document.getElementById("enter_originReal")! as HTMLInputElement;
      f.value = pos.origin.re.toString();
      f = document.getElementById("enter_originImag")! as HTMLInputElement;
      f.value = pos.origin.im.toString();
    } else {
      f = document.getElementById("enter_centreReal")! as HTMLInputElement;
      f.value = pos.centre().re.toString();
      f = document.getElementById("enter_centreImag")! as HTMLInputElement;
      f.value = pos.centre().im.toString();
    }

    // clear out: Axes Im, Zoom
    ["axesImag", "zoom"].forEach(f => {
      let field = document.getElementById("enter_" + f)! as HTMLInputElement;
      if (field !== null) {
        field.value = "";
      }
    });
  }

  origin_is_currently_visible() : boolean {
    let originDisplay = document.getElementById("show-origin");
    return !originDisplay?.classList.contains("hidden");
  }

  toggle_origin_centre() {
    if (this.origin_is_currently_visible()) {
      document.getElementById("show-origin")?.classList.add("hidden");
      document.getElementById("show-centre")?.classList.remove("hidden");
      document.getElementById("enter-origin")?.classList.add("hidden");
      document.getElementById("enter-centre")?.classList.remove("hidden");
      // Clear out fields we just hid
      let field = document.getElementById("enter_originReal") as HTMLInputElement;
      field.value = "";
      field = document.getElementById("enter_originImag") as HTMLInputElement;
      field.value = "";
    } else {
      document.getElementById("show-origin")?.classList.remove("hidden");
      document.getElementById("show-centre")?.classList.add("hidden");
      document.getElementById("enter-origin")?.classList.remove("hidden");
      document.getElementById("enter-centre")?.classList.add("hidden");
      // Clear out fields we just hid
      let field = document.getElementById("enter_centreReal") as HTMLInputElement;
      field.value = "";
      field = document.getElementById("enter_centreImag") as HTMLInputElement;
      field.value = "";
    }
  }

  // dummy function to shut up a linter warning in main.ts
  noop() { }
}
