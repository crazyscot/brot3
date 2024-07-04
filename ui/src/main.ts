// brot3 main UI
// (c) 2024 Ross Younger

import "@fontsource/inter/400.css"
import "@fontsource/inter/700.css"
import "@fontsource/inter/900.css"
import "@fontsource/roboto/400.css"
import "@fontsource/roboto/700.css"
import "@fontsource/roboto/900-italic.css"
import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'

import { About } from './about.ts'
import { ErrorHandler } from './error_handler.ts'
import { HeadsUpDisplay } from './hud.ts'
import { IterationLimitBox } from './max_iter.ts'
import { Menu } from './menu.ts'
import { SaveSizeBox } from './save_size.ts'
import { SelectionOverlay } from './selection_overlay.tsx'
import { Viewer } from './viewer.ts'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
<div id="topbar">
  <span class="busy">
    <em>busy</em>
  </span>
</div>
${HeadsUpDisplay.top_html}
<div id="openseadragon">
${About.html}
<!-- Trap: Modals not within this div won't be cloned into fullscreen mode -->
</div>
<div id="bottombar">
${IterationLimitBox.html}
${SaveSizeBox.html}
${HeadsUpDisplay.bottom_html}
${SelectionOverlay.html}
</div>
`;


class Brot3UI {
  private errorHandler: ErrorHandler = new ErrorHandler();
  private viewer: Viewer = new Viewer();
  private saveSizeBox: SaveSizeBox;
  private maxIterBox: IterationLimitBox;
  constructor(doc: Document) {
    this.errorHandler.bind_events();
    this.setupWindow();
    this.saveSizeBox = new SaveSizeBox(doc, this.viewer);
    this.maxIterBox = new IterationLimitBox(doc, this.viewer);
    new SelectionOverlay(doc, this.viewer);
    new Menu(doc, this.viewer, this.saveSizeBox, this.maxIterBox);
  }

  async setupWindow() {
    getVersion().then(ver => appWindow.setTitle(`brot3 ${ver}`));
  }
}

new Brot3UI(document);
