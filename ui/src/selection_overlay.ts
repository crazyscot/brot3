// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { ClickEventListener, element_is_displayed, toggle_tr_visibility } from './dom_util'

export class SelectionOverlay {
    static readonly html: string = `
    <div class="selector modal" id="selector-panel">
      <div class="modal-content">
        <span class="close" id="close-selector">&times;</span>
        <h1>Hello, selector world!</h1>
        </div>
    </div>
`;

    panel: HTMLElement | null;
    closer: ClickEventListener;

    constructor(doc: Document) {
        let self = this; // For closures
        this.panel = doc.querySelectorAll('#selector-panel')[0] as HTMLElement;

        // Hide by default
        if (element_is_displayed(this.panel!)) {
            self.toggle_visibility();
            console.log("was visible");
        }

        this.closer = new ClickEventListener(
            document.getElementById("close-selector")!,
            function (_event: Event) {
                self.toggle_visibility();
            }
        );
    }

    toggle_visibility() {
        toggle_tr_visibility(this.panel!);
    }

    select_fractal() {
        this.toggle_visibility();
    }

    noop() { }
}
