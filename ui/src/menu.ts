import './menu.css'
import '../node_modules/material-icons/iconfont/filled.css'

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
    dropdown: Element | null;

    constructor(doc: Document) {
        let self = this; // for closures
        this.doc = doc;
        let element = doc.querySelector('#menu');
        element!.innerHTML = MENU_HTML;
        this.dropdown = doc.querySelector('#menu-inner');
        this.dropdown?.appendChild(doc.createElement("li"))

        // Bind Click event to the drop down navigation button
        doc.querySelector('.nav-button')!.addEventListener('click', function () {
          /*  Toggle the CSS closed class which reduces the height of the UL thus
              hiding all LI apart from the first */
            self.dropdown!.classList.toggle('closed');
        }, false);
        this.build();
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
        this.addItem(new MenuItem("foo", function (_ev: MouseEvent) { console.log("foo!!"); }));
    }

    noop() { }
}
