// brot3 UI/menu elements & event handling
// (c) 2024 Ross Younger

import './menu.css'
import '../node_modules/material-icons/iconfont/filled.css'

import { listen } from '@tauri-apps/api/event'

import { About } from './about.ts'

// Twin of rust menu::DisplayMessageDetail
class DisplayMessageDetail {
    what: string;
    constructor(what: string) {
        this.what = what;
    }
}

export class Menu {
    doc: Document;
    about: About;
    zoom_panel: HTMLElement | null;
    position_panel: HTMLElement[];
    position_entry_rows: HTMLElement[];

    constructor(doc: Document) {
        let self = this; // for closures
        this.doc = doc;
        this.zoom_panel = doc.querySelector('#zoom-panel') as HTMLElement;
        this.position_panel = Array.from(doc.querySelectorAll('.position-panel'), e => e as HTMLElement);
        this.position_entry_rows = Array.from(doc.querySelectorAll('.position-entry'), e => e as HTMLElement);

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
                case "toggle_zoom":
                    this.toggle_tr_visibility(this.zoom_panel!);
                    break;
                case "toggle_position":
                    this.position_panel.forEach(e => this.toggle_tr_visibility(e));
                    break;
                case "go_to_position":
                    this.position_entry_rows.forEach(e => this.toggle_tr_visibility(e));
                    break;
                default:
                    console.error(`unknown display_message detail ${event.payload.what}`);
            }
        });
    }

    toggle_tr_visibility(e: HTMLElement) {
        if (e.style.display === "none" || e.style.display === "") {
            e.style.display = "table-row";
        } else {
            e.style.display = "none";
        }
    }

    noop() { }
}
