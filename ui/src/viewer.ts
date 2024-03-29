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

class HeadsUpDisplay {
  zoom: Element | null;
  originReal: Element | null;
  originImag: Element | null;
  axesReal: Element | null;
  axesImag: Element | null;
  constructor(doc : Document) {
    this.zoom = doc.querySelectorAll('#zoom')[0];
    this.originReal = doc.querySelectorAll('#originReal')[0];
    this.originImag = doc.querySelectorAll('#originImag')[0];
    this.axesReal = doc.querySelectorAll('#axesReal')[0];
    this.axesImag = doc.querySelectorAll('#axesImag')[0];
  }
  update(zoom: number, origin: EnginePoint, axes: EnginePoint) {
    this.zoom!.innerHTML = `${zoom.toPrecision(4)}`;
    this.axesReal!.innerHTML = maybe_leading("&nbsp;", axes.re);
    this.axesImag!.innerHTML = maybe_leading("+", axes.im);
    this.originReal!.innerHTML = maybe_leading("&nbsp;", origin.re);
    this.originImag!.innerHTML = maybe_leading("+", origin.im);
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
    var updateIndicator = function() {
      let vp = viewer.viewport;
      var zoom : number = vp.getZoom(true);
      //var imageZoom = vp.viewportToImageZoom(zoom);
      // We know that top left is webPoint 0,0; bottom right is W-1,H-1.
      // These are the web (pixel) coordinates.
      var topLeft = new OpenSeadragon.Point(0, 0);
      var bottomRight = new OpenSeadragon.Point(self.width - 1, self.height - 1);
      // Convert to viewport coordinates:
      var topLeftView = vp.pointFromPixelNoRotate(topLeft);
      var bottomRightView = vp.pointFromPixelNoRotate(bottomRight);

      // Bottom Left is the origin (as mathematicians would call it, not computer images!)
      var originView = new OpenSeadragon.Point(topLeftView.x, bottomRightView.y);

      // Axes := BR - TL
      var axesLengthView = bottomRightView.minus(topLeftView);

      // Convert to complex
      let meta = self.current_metadata; // Caution, closure capture!
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
      self.hud.update(zoom, originComplex, axesComplex);
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
        updateIndicator();
      })
      .catch((e) => {
        console.log(`Error retrieving metadata: ${e}`);
      }
    );
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

  go_to(destination: Map<string, number>) {
  }

  // dummy function to shut up a linter warning in main.ts
  noop() { }
}
