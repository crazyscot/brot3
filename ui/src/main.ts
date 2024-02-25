// brot3 main UI
// (c) 2024 Ross Younger

import './style.css'
import { getVersion } from '@tauri-apps/api/app'
import { appWindow } from '@tauri-apps/api/window'
import jQuery from 'jquery'
import { Viewer } from './viewer.ts'

document.querySelector<HTMLDivElement>('#main')!.innerHTML = `
<div id="seadragon-viewer"></div>
`;

async function setupWindow() {
  // Dynamically size to fill the window
  let viewerElement = jQuery('#seadragon-viewer');
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
gViewer.noop();