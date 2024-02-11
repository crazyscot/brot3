import './style.css'
import { invoke } from '@tauri-apps/api'
import OpenSeadragon from 'openseadragon'
import jQuery from 'jquery'

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

class TileSpec {
  level: number;
  dx: number;
  dy: number;
  width: number;
  height: number;
  constructor(data: TilePostData, width: number, height: number) {
    this.level = data.level;
    this.dx = data.dx;
    this.dy = data.dy;
    this.width = width;
    this.height = height;
  }
}

const TILE_SIZE = 256;
const IMAGE_DIMENSION = 1024 * 1024;

var viewer = OpenSeadragon({
  id:         "seadragon-viewer",
  prefixUrl: "/openseadragon/images/",
  homeFillsViewer: true,
  autoResize: true,
  preserveImageSizeOnResize:true,
  visibilityRatio: 1.0,
  debugMode: true,
  tileSources: {
    height: IMAGE_DIMENSION,
    width: IMAGE_DIMENSION,
    tileSize: TILE_SIZE,
    minLevel: 9,
    tileOverlap: 1,
    getTileUrl: function (level, x, y) {
      // TODO add fractal, colour (or we'll break cacheing!)
      return `${level}/${x}-${y}`;
    },
    // caution: @types/openseadragon 3.0.10 doesn't know about these functions
    getTilePostData: function (level: number, x: number, y: number) {
      // TODO add fractal, colour
      return new TilePostData(level, x, y);
    },
    downloadTileStart: function (context) {
      // TODO shell out to rust. we can put our stuff (e.g. memos) in userData. call ctx.finish() when done.
      // A queue of outstanding jobs, parallelisable? Manage these in Rust, I think.
      // See https://tauri.app/v1/guides/features/command : they can be async.
      // All returns must be serde::Serialize.
      // looks like commands are async to JS. Are they allowed to block? They can be async.

      // Data for Rust:
      // Fractal, Colourer, Level/X/Y [we'll have rust figure it out]
      //console.log(`DLTS ${context.postData}`);
      // tile dx and dy are the column and row numbers FOR THE ZOOM LEVEL.
      // Given 1048576x1048576 pixels, we start at level 10 (4x4 tiles comprise the image) and end at level 20 (4096x4096)
      // => At zoom level X, the image is 2^X pixels across.

      // context.postData...

      let data = context.userData;
      invoke('tile', {
        spec: new TileSpec(context.postData, TILE_SIZE, TILE_SIZE)
      })
        .then((response) => {
          // "convert the data to a canvas and return its 2D context"
          // response.blob is a byte array
          let blob = new Uint8ClampedArray(response.blob);
          let image = new ImageData(blob, TILE_SIZE, TILE_SIZE, { "colorSpace": "srgb" });
          let canvas = document.createElement("canvas");
          let ctx2d = canvas.getContext("2d");
          ctx2d?.putImageData(image, 0, 0);
          context.finish(ctx2d);
        })
        .catch((e) => {
          context.finish(null, null, e.toString());
        });
    },
    downloadTileAbort: function (context) {
      // TODO halt (remove from queue?) // This is a Rust call.
    },
    createTileCache: function (cache, data) {
      cache._data = data;
    },
    destroyTileCache: function (cache) {
      cache._data = null;
    },
    getTileCacheData: function(cache) {
      return cache._data;
    },
    getTileCacheDataAsImage: function() {
      // not implementing all the features brings limitations to the
      // system, namely tile.getImage() will not work and also
      // html-based drawing approach will not work
      throw "getTileCacheDataAsImage not implemented";
    },

    getTileCacheDataAsContext2D: function(cache) {
      // our data is already context2D - what a luck!
      return cache._data;
    }
  },
});


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
