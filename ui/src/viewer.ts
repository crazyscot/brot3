// brot3 fractal viewer (bridge to OpenSeadragon)
// (c) 2024 Ross Younger

import { invoke } from '@tauri-apps/api'
import { UnlistenFn, listen } from '@tauri-apps/api/event'
import jQuery from 'jquery'
import OpenSeadragon from 'openseadragon'

import { SerialAllocator } from './serial_allocator'
import { TileSpec, TileResponse, TileError, TilePostData } from './engine_types'

var gSerial = new SerialAllocator();
const TILE_SIZE = 128;
const IMAGE_DIMENSION = 1024 * 1024 * 1024 * 1024;

export class Viewer {
    osd: any | null;  // OpenSeadragon.Viewer
    redraw_event: number | undefined; // setTimeout / clearTimeout
    unlisten_tile_complete: UnlistenFn | null = null;
    unlisten_tile_error: UnlistenFn | null = null;
    outstanding_requests: Map<number, any/*OpenSeadragon.ImageJob*/> = new Map();

    constructor() {
        let self = this; // Closure helper

        this.osd = OpenSeadragon({
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
        });

        this.bind_events().then(() => {
          this.osd.addHandler("before-destroy", function () {
            self.unlisten_tile_complete?.();
            self.unlisten_tile_error?.();
          });
        });

        // Window resize
        // Rather than caning the system as we get a resize event for every pixel, add a slight debounce
        window.addEventListener('resize', function (_event) {
          if (self.redraw_event !== undefined) {
            this.clearTimeout(self.redraw_event);
          }
          self.redraw_event = this.setTimeout(function () {
            let viewerElement = jQuery('#seadragon-viewer');
            console.log(`resizing to ${window.innerWidth} x ${window.innerHeight}`);
            viewerElement.height(window.innerHeight);
            viewerElement.width(window.innerWidth);
            self.osd.viewport.applyConstraints();
          }, 100);
        }, true);
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

    // dummy function to shut up a linter warning in main.ts
    noop() { }
}