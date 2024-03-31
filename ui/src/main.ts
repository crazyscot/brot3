// brot3 main UI
// (c) 2024 Ross Younger

import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'

import { About } from './about.ts'
import { HeadsUpDisplay } from './hud.ts'
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
${HeadsUpDisplay.html}
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
