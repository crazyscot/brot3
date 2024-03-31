// brot3 UI/menu elements & event handling
// (c) 2024 Ross Younger

import './menu.css'
import '../node_modules/material-icons/iconfont/filled.css'

import { listen } from '@tauri-apps/api/event'

import { About } from './about.ts'
import { Viewer } from './viewer.ts'

// Twin of rust menu::DisplayMessageDetail
class DisplayMessageDetail {
    what: string;
    constructor(what: string) {
        this.what = what;
    }
}

// Fields we read from the Enter Position form.
// N.B. Fields in the DOM are prefixed with "enter_". These field names are in the output object.
// Not all fields need be present; origin OR centre, plus one of zoom/axesReal/axesImag.
const position_entry_fields = [
    "zoom",
    "centreReal",
    "centreImag",
    "originReal",
    "originImag",
    "axesReal",
    "axesImag",
];

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
            self.action_go_to_position();
        }
        doc.getElementById("action_copy_current_position")!.onclick = function (_e) {
            self.action_copy_current_position();
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
                    this.viewer.toggle_hud();
                    break;
                case "go_to_position":
                    this.viewer.toggle_position_entry_panel()
                    break;
                case "toggle_origin_centre":
                    this.viewer.toggle_origin_centre();
                    break;
                default:
                    console.error(`unknown display_message detail ${event.payload.what}`);
            }
        });
    }

    action_go_to_position() {
        let destination = this.parse_entered_position();
        this.viewer.go_to(destination);
    }

    action_copy_current_position() {
        this.viewer.copy_current_position();
    }

    parse_entered_position() : Map<string, number> {
        let result = new Map<string, number>();
        let errors = new Array<string>;
        for (let f of position_entry_fields) {
            let fieldId = "#enter_" + f;
            let fieldElement = this.doc.querySelector<HTMLInputElement>(fieldId);
            if (fieldElement === null) {
                // quietly ignore it
                continue;
            }
            let value = parseFloat(fieldElement.value);
            // this results in NaN if a field is empty; that's OK as not all are mandatory. Viewer will figure it out.
            result.set(f, value);
        };
        if (errors.length !== 0) {
            let message = `Form data error: ${errors.join(", ")}`;
            throw new Error(message);
        }
        return result;
    }

    noop() { }
}
