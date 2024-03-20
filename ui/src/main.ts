// brot3 main UI
// (c) 2024 Ross Younger

import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'
import jQuery from 'jquery'

import { About } from './about.ts'
import { Menu } from './menu.ts'
import { Viewer } from './viewer.ts'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
<div id="topbar">
  <span class="right-align">
    <nav id="menu"></nav>
  </span>
</div>
<div id="openseadragon">
${About.html}
<!-- Trap: Modals not within this div won't be cloned into fullscreen mode -->
</div>
<div id="bottombar">
  <div class="info">
  <table>
    <tr><th>Zoom:</th><td id="zoom"></td></tr>
    <tr><th>Origin:</th><td><span id="originReal"></span>,</td><td><span id="originImag"></span>i</td></tr>
    <tr><th>Axes:</th><td><span id="axesReal"></span>,</td><td><span id="axesImag"></span>i</td></tr>
  </table>
  </div>
</div>
`;

let gViewer = new Viewer();

async function setupWindow() {
  gViewer.resize();
  getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));
}

setupWindow();

let gMenu = new Menu(document);
gViewer.noop();
gMenu.noop();
