import './style.css'
import { invoke } from '@tauri-apps/api'
import OpenSeadragon from 'openseadragon'
import $ from 'jquery'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
<div id="seadragon-viewer"></div>
`;
// Dynamically size to fill the window. This is also used to resize.
let viewerElement = $('#seadragon-viewer');
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

var duomo = {
  Image: {
    xmlns: "http://schemas.microsoft.com/deepzoom/2008",
    Url: "//openseadragon.github.io/example-images/duomo/duomo_files/",
    Format: "jpg",
    Overlap: "2",
    TileSize: "256",
    Size: {
      Width:  "13920",
      Height: "10200"
    }
  }
};

var viewer = OpenSeadragon({
  id: "seadragon-viewer",
  prefixUrl: "//openseadragon.github.io/openseadragon/images/",
  tileSources: duomo,
  homeFillsViewer: true,
  autoResize: true,
  preserveImageSizeOnResize:true,
  visibilityRatio: 1.0,
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
