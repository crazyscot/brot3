import './style.css'
import { invoke } from '@tauri-apps/api'
import OpenSeadragon from 'openseadragon'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
  <div id="seadragon-viewer" style="width: ${window.innerWidth}px; height: ${window.innerHeight}px;"></div>
`

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
  homeFillsViewer: true
});
