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
  <span id="info-display">
    <-- Bottom line info bar -->
  </span>
</div>
`;

async function setupWindow() {
  // Dynamically size to fill the window
  let viewerElement = jQuery('#openseadragon');
  if (viewerElement.height() == 0) {
    viewerElement.height(window.innerHeight);
  }
  if (viewerElement.width() == 0) {
    viewerElement.width(window.innerWidth);
  }
  console.log(`Window resized to ${window.innerWidth} x ${window.innerHeight}`);

  getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));
}

setupWindow();

let gViewer = new Viewer();
let gMenu = new Menu(document);
gViewer.noop();
gMenu.noop();
