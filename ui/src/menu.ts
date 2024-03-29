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
// Not all fields need be present; origin OR centre, plus one of zoom/axisReal/axisImag.
const position_entry_fields = [
    "zoom",
    "originReal",
    "originImag",
    "axisReal",
    "axisImag",
];

export class Menu {
    doc: Document;
    viewer: Viewer;
    about: About;
    position_display: HTMLElement[];
    position_entry_rows: HTMLElement[];

    constructor(doc: Document, viewer: Viewer) {
        let self = this; // for closures
        this.doc = doc;
        this.viewer = viewer;

        this.position_display = Array.from(doc.querySelectorAll('tr.position-display'), e => e as HTMLElement);

        // Position entry form
        this.position_entry_rows = Array.from(doc.querySelectorAll('tr.position-entry'), e => e as HTMLElement);
        // Hide the form by default
        this.position_entry_rows.forEach(e => this.toggle_tr_visibility(e));
        doc.getElementById("form_go_to_position")!.onsubmit = function (e) {
            e.preventDefault();
            self.action_go_to_position();
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
                    this.position_display.forEach(e => this.toggle_tr_visibility(e));
                    break;
                case "go_to_position":
                    let visible = false;
                    this.position_entry_rows.forEach(e => visible = this.toggle_tr_visibility(e));
                    if (visible) {
                        let element = document.getElementById(`enter_originReal`) as HTMLInputElement;
                        element!.focus();
                        element!.select();
                    }
                    break;
                default:
                    console.error(`unknown display_message detail ${event.payload.what}`);
            }
        });
    }

    toggle_tr_visibility(e: HTMLElement) : boolean {
        if (e.style.display === "none") {
            e.style.display = "table-row";
            return true;
        } else {
            e.style.display = "none";
            return false;
        }
    }

    action_go_to_position() {
        let destination = this.parse_entered_position();
        this.viewer.go_to(destination);
    }

    parse_entered_position() : Map<string, number> {
        let result = new Map<string, number>();
        let errors = new Array<string>;
        for (let f of position_entry_fields) {
            let fieldId = "#enter_" + f;
            let fieldElement = this.doc.querySelector<HTMLInputElement>(fieldId);
            if (fieldElement === null) {
                errors.unshift(`missing HTML element ${fieldId}`);
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
