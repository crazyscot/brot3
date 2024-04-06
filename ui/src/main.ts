// brot3 main UI
// (c) 2024 Ross Younger

import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'

import { About } from './about.ts'
import { ErrorHandler } from './error_handler.ts'
import { HeadsUpDisplay } from './hud.ts'
import { IterationLimitBox } from './max_iter.ts'
import { Menu } from './menu.ts'
import { SaveSizeBox } from './save_size.ts'
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
${IterationLimitBox.html}
${SaveSizeBox.html}
${HeadsUpDisplay.html}
</div>
`;

let gErrorHandler = new ErrorHandler();
gErrorHandler.bind_events();

let gViewer = new Viewer();

async function setupWindow() {
  gViewer.resize();
  getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));
}
setupWindow();

let gSaveSizeBox = new SaveSizeBox(document, gViewer);
let gMaxIter = new IterationLimitBox(document, gViewer);
let gMenu = new Menu(document, gViewer, gSaveSizeBox, gMaxIter);
gMenu.noop();
