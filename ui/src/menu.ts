// brot3 UI/menu elements & event handling
// (c) 2024 Ross Younger

import '../node_modules/material-icons/iconfont/filled.css'

import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'

import { About } from './about.ts'
import { RenderSpec } from './engine_types.ts'
import { Viewer } from './viewer.ts'

// Twin of rust menu::DisplayMessageDetail
class DisplayMessageDetail {
    what: string;
    constructor(what: string) {
        this.what = what;
    }
}

export class Menu {
    doc: Document;
    viewer: Viewer;
    about: About;

    constructor(doc: Document, viewer: Viewer) {
        let self = this; // for closures
        this.doc = doc;
        this.viewer = viewer;

        // Bind form actions
        doc.getElementById("form_go_to_position")!.onsubmit = function (e) {
            e.preventDefault();
            let destination = self.viewer.hud.parse_entered_position();
            self.viewer.go_to_position(destination);
        }
        doc.getElementById("action_copy_current_position")!.onclick = function (_e) {
            self.viewer.copy_current_position();
        }

        this.about = new About(self.doc.getElementById("aboutModal")!);
        this.bind_events();
    }

    async bind_events() {
        let self = this;
        await listen<DisplayMessageDetail>('display_message', (event) => {
            switch (event.payload.what) {
                case "show_about":
                    self.about!.show();
                    break;
                case "toggle_position":
                    this.viewer.hud.toggle_visibility();
                    break;
                case "go_to_position":
                    this.viewer.hud.toggle_position_entry_panel()
                    break;
                case "toggle_origin_centre":
                    this.viewer.hud.toggle_origin_centre();
                    break;
                case "save_image":
                    let position = this.viewer.get_position();
                    invoke('save_image_workflow', {
                        spec: new RenderSpec(position.origin, position.axes_length, this.viewer.width, this.viewer.height)
                    });
                    break;
                default:
                    console.error(`unknown display_message detail ${event.payload.what}`);
            }
        });
    }

    noop() { }
}
