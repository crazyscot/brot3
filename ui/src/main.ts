// brot3 main UI
// (c) 2024 Ross Younger

import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'

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
    <tr class="zoom-display"><th>Zoom:</th><td id="zoom"></td></tr>
    <tr class="position-display"><th>Origin:</th><td><span id="originReal"></span></td><td><span id="originImag"></span>i</td></tr>
    <tr class="position-display"><th>Axes:</th><td><span id="axesReal"></span></td><td><span id="axesImag"></span>i</td></tr>
    <tr class="position-entry"><th>Zoom:</th><td colspan="2"><input type="text" id="enter_zoom" /></td></tr>
    <tr class="position-entry"><th>Origin:</th><td><input type="text" id="enter_originReal" /></td><td>+ <input type="text" id="enter_originImg" /> i</td></tr>
    <tr class="position-entry"><th>Axes:</th><td><input type="text" id="enter_axisReal" /></td><td>+ <input type="text" id="enter_originReal" /> i</td></tr>
    <tr class="position-entry"><th><input type="submit" id="action_go_to_position" value="Go"</th><td colspan="2"/></tr>
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
