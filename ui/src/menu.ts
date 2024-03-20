// brot3 JS menu
// (c) 2024 Ross Younger

import './menu.css'
import '../node_modules/material-icons/iconfont/filled.css'

import { listen } from '@tauri-apps/api/event'

import { About } from './about.ts'

const MENU_HTML = `
<ul class="drop-down closed" id="menu-inner">
<li><a href="#" class="nav-button"><span class="material-icons">menu</span></a></a></li>
</ul>
`;

type eventCallback = (event: MouseEvent) => void;

class MenuItem {
    display: string;
    class: string;
    onClick: eventCallback;
    constructor(disp: string, onClick: eventCallback) {
        this.display = disp;
        this.class = "menu-" + disp;
        this.onClick = onClick;
    }
}

export class Menu {
    doc: Document;
    about: About;
    dropdown: HTMLElement | null;
    zoom_panel: HTMLElement | null;

    constructor(doc: Document) {
        let self = this; // for closures
        this.doc = doc;
        let element = doc.querySelector('#menu');
        element!.innerHTML = MENU_HTML;
        this.dropdown = doc.querySelector('#menu-inner');
        this.dropdown?.appendChild(doc.createElement("li"))
        this.zoom_panel = doc.querySelector('#zoom-panel');

        // Bind Click event to the drop down navigation button
        doc.querySelector('.nav-button')!.addEventListener('click', function () {
          /*  Toggle the CSS closed class which reduces the height of the UL thus
              hiding all LI apart from the first */
            self.dropdown!.classList.toggle('closed');
        }, false);

        this.about = new About(self.doc.getElementById("aboutModal")!);

        this.build();
        this.bind_events();
    }

    addItem(item: MenuItem) {
        let self = this;
        let li = this.doc.createElement("li");
        let anchor = this.doc.createElement("a");
        li.appendChild(anchor);
        anchor.textContent = item.display;
        anchor.classList.add(item.class);
        let attrib = document.createAttribute("href");
        attrib.value = '#';
        anchor.attributes.setNamedItem(attrib);
        this.dropdown?.appendChild(li);
        anchor.addEventListener('click', function (this: HTMLAnchorElement, event: MouseEvent) {
            item.onClick(event);
            // Close the menu
            self.dropdown!.classList.add('closed');
        });
    }

    build() {
        let self = this;
        this.addItem(new MenuItem("About", function (_ev: MouseEvent) {
            self.about!.show();
        }));
    }

    async bind_events() {
        let self = this;
        await listen<void>('showAbout', (_event) => {
            self.about!.show();
        });
        await listen<void>('toggle_zoom', (_event) => {
            this.toggle_tr_visibility(this.zoom_panel!);
        });
    }

    toggle_tr_visibility(e: HTMLElement) {
        if (e.style.display === "none") {
            e.style.display = "table-row";
        } else {
            e.style.display = "none";
        }
    }

    noop() { }
}
