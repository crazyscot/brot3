import './style.css'
import typescriptLogo from './typescript.svg'
import viteLogo from '/vite.svg'
import { setupCounter } from './counter.ts'
import { invoke } from '@tauri-apps/api'
import OpenSeadragon from 'openseadragon'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <h1>Vite + TypeScript</h1>
    <div class="card">
      <button id="counter" type="button"></button>
    </div>
  </div>
  <div id="openseadragon1" style="width: 400px; height: 400px;"></div>
`

setupCounter(document.querySelector<HTMLButtonElement>('#counter')!)

// now we can call our Command!
// Right-click the application background and open the developer tools.
// You will see "Hello, World!" printed in the console!
invoke('greet', { name: 'World' })
  // `invoke` returns a Promise
  .then((response) => console.log(response))

var viewer = OpenSeadragon({
    id: "openseadragon1",
    prefixUrl: "/openseadragon/images/",
    tileSources: "/path/to/my/image.dzi"
});