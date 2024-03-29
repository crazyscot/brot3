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
  <form id="form_go_to_position">
    <table>
      <tr class="position-display"><th>Origin:</th><td><span id="originReal"></span></td><td><span id="originImag"></span>i</td></tr>
      <tr class="position-display"><th>Axes:</th><td><span id="axesReal"></span></td><td><span id="axesImag"></span>i</td><th>Zoom:</th><td id="zoom"></td></tr>
      <tr class="position-entry"><th>Origin:</th><td><input type="text" id="enter_originReal" /></td><td>+ <input type="text" id="enter_originImag" /> i</td><td colspan="3"/></tr>
      <tr class="position-entry"><th>Axes:</th><td><input type="text" id="enter_axisReal" /> real</td><td><em>or</em> <input type="text" id="enter_axisImag" /> im</td>
        <td><em>or</em> Zoom:</td><td><input type="text" id="enter_zoom" /></td><td><input type="submit" id="action_go_to_position" value="Go"></td></tr>
    </table>
  </form>
  </div>
</div>
`;

let gViewer = new Viewer();

async function setupWindow() {
  gViewer.resize();
  getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));
}

setupWindow();

let gMenu = new Menu(document, gViewer);
gViewer.noop();
gMenu.noop();
