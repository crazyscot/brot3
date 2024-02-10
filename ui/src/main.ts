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
  console.log(`H set to ${viewerElement.height()}`)
}
if (viewerElement.width() == 0) {
  viewerElement.width(window.innerWidth);
  console.log(`W set to ${viewerElement.width()}`)
}

// now we can call our Command!
// Right-click the application background and open the developer tools.
// You will see "Hello, World!" printed in the console!
invoke('greet', { name: 'World' })
  // `invoke` returns a Promise
  .then((response) => console.log(response))

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

      let data = context.userData;
      data.image = new Image(1, 1);
      context.finish(data.image);

      // NEXT: Rust should call back ASYNC when there is image data.
    },
    downloadTileAbort: function (context) {
      // TODO halt (remove from queue?) // This is a Rust call.
    },
    // TODO tileCache functions?
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
