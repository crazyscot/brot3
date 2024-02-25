import './style.css'
import { invoke } from '@tauri-apps/api'
import { getVersion } from '@tauri-apps/api/app'
import { listen } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'
import OpenSeadragon from 'openseadragon'
import jQuery from 'jquery'
import { SerialAllocator } from './serial_allocator'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
<div id="seadragon-viewer"></div>
`;
// Dynamically size to fill the window. This is also used to resize.
let viewerElement = jQuery('#seadragon-viewer');
if (viewerElement.height() == 0) {
  viewerElement.height(window.innerHeight);
}
if (viewerElement.width() == 0) {
  viewerElement.width(window.innerWidth);
}
console.log(`Window size is ${window.innerWidth} x ${window.innerHeight}`);

getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));

class TilePostData {
  dx: number;
  dy: number;
  level: number;
  constructor(l: number, x: number, y: number) {
    this.dx = x;
    this.dy = y;
    this.level = l;
  }
  toString(): string {
    return `${this.level}/${this.dx}-${this.dy}`;
  }
};

var gSerial = new SerialAllocator();

/// Twin of Rust ViewerTileSpec struct. This class is also the userData element of ImageJob.userData.
class TileSpec {
  // TODO: fractal, colourer
  serial: number;
  level: number;
  dx: number;
  dy: number;
  width: number;
  height: number;
  constructor(serial: number, data: TilePostData, width: number, height: number) {
    this.serial = serial; // Always obtain from gSerial.next() !
    this.level = data.level;
    this.dx = data.dx;
    this.dy = data.dy;
    this.width = width;
    this.height = height;
  }
}

class TileResponse {
  serial: number;
  rgba_blob: Uint8Array;
  constructor() {
    this.serial = 0;
    this.rgba_blob = new Uint8Array();
  }
}

const TILE_SIZE = 128;
const IMAGE_DIMENSION = 1024 * 1024 * 1024 * 1024;

let outstanding_requests = new Map<number, any/*OpenSeadragon.ImageJob*/>();

const unlisten_tile_complete = await listen<TileResponse>('tile_complete', (event) => {
  let response: TileResponse = event.payload;
  let context = outstanding_requests.get(response.serial);
  let spec:TileSpec = context.userData;
  console.log(`got tile #${response.serial} = ${spec.level}/${spec.dx}-${spec.dy}`);

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
});
// Note the before-destroy handler we set up below.

var viewer = OpenSeadragon({
  id:         "seadragon-viewer",
  prefixUrl: "/openseadragon/images/",
  homeFillsViewer: true,
  autoResize: true,
  preserveImageSizeOnResize: true,
  visibilityRatio: 1.0,
  debugMode: false,
  showRotationControl: false,
  rotationIncrement: 15,

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

      let spec = new TileSpec(await gSerial.next(), context.postData, TILE_SIZE, TILE_SIZE);
      context.userData = spec;
      outstanding_requests.set(spec.serial, context);
      invoke('start_tile', {
        spec: spec
      })
      .catch((e) => {
        context.finish(null, null, e.toString());
      });
    },
    downloadTileAbort: function (context: any /*OpenSeadragon.ImageJob*/) {
      console.log(`OSD abort: tile #${context.userData.serial}`);
      invoke('abort_tile', { serial: context.userData.serial })
      .catch((e) => {
        context.finish(null, null, e.toString());
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
});

viewer.addHandler("before-destroy", function () { unlisten_tile_complete(); });

// Rather than caning the system as we get a resize event for every pixel, add a slight debounce
let redrawer: number|undefined = undefined;

window.addEventListener('resize', function (_event) {
  if (redrawer !== undefined) {
    this.clearTimeout(redrawer);
  }
  redrawer = setTimeout(function () {
    console.log(`resizing to ${window.innerWidth} x ${window.innerHeight}`);
    viewerElement.height(window.innerHeight);
    viewerElement.width(window.innerWidth);
    viewer.viewport.applyConstraints();
  }, 100);
}, true);
