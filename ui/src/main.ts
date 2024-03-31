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
  <div class="info" id="info-panel">
  <form id="form_go_to_position">
    <table>
      <tr class="position-display"><td rowspan="4"><span class="close" id="close-hud">&times;</span></td></tr>
      <tr class="position-display hidden" id="show-origin"><th>Origin:</th><td><span id="originReal"></span></td><td><span id="originImag"></span> i</td></tr>
      <tr class="position-display" id="show-centre"><th>Centre:</th><td><span id="centreReal"></span></td><td><span id="centreImag"></span> i</td></tr>
      <tr class="position-display">
        <th>Axes:</th><td><span id="axesReal"></span> re,</td><td><span id="axesImag"></span> im</td>
      </tr>
      <tr class="position-display">
        <th>Zoom:</th><td id="zoom"></td>
      </tr>
      <!--------------------------------------------->
      <tr class="position-entry"><td rowspan="5"><span class="close" id="close-entry">&times;</span></td></tr>
      <tr class="position-entry hidden" id="enter-origin">
        <th>Origin:</th>
        <td><input type="text" id="enter_originReal" /></td>
        <td>+ <input type="text" id="enter_originImag" /> i</td>
        <td colspan="3"/>
      </tr>
      <tr class="position-entry" id="enter-centre">
        <th>Centre:</th>
        <td><input type="text" id="enter_centreReal" /></td>
        <td>+ <input type="text" id="enter_centreImag" /> i</td>
        <td colspan="3"/>
      </tr>
      <tr class="position-entry">
        <th>Axis:</th>
        <td colspan="2"><input type="text" id="enter_axesReal" /> real</td>
        <td colspan="2"/>
      </tr>
      <tr class="position-entry">
        <td />
        <td><button type="submit" id="action_go_to_position">Go</button></td>
        <td><button type="button" id="action_copy_current_position">Copy Current</button></td>
        <td colspan="2" />
      </tr>
      <tr class="position-entry"><td colspan="6" id="position-error-text" /></tr>
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
