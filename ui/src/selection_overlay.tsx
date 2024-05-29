// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { element_is_displayed, toggle_element_visibility } from './dom_util'
import { createRoot, Root } from 'react-dom/client';
import React from 'react';
import { effectModalClickOrEscape } from './modal-react';

const SelectionModal = ({ doHide = () => { } }) => {
    const hide = () => {
        doHide();
    };

    const ref = effectModalClickOrEscape(() => {
        hide();
    });
    return (
        <div ref={ref}>
            <div className="modal-content">
                <span className="close" id="close-selector" onClick={hide}>&times;</span>
                <h3>Select Fractal</h3>
                <h1>TODO - populate list</h1>
            </div>
        </div>
    );
};

export class SelectionOverlay {
    // Our base html, which is a React root
    static readonly html: string = `<div class="selector modal" id="selector-panel"></div>`;

    panel: HTMLElement | null;
    readonly root: Root;

    constructor(doc: Document) {
        let self = this; // For closures
        this.panel = doc.querySelectorAll('#selector-panel')[0] as HTMLElement;

        // Hide by default
        self.set_visibility(false);
        this.root = createRoot(this.panel);
        this.root.render(<SelectionModal doHide={() => { this.set_visibility(false) }} />);
    }

    set_visibility(visible: boolean) {
        if (visible !== element_is_displayed(this.panel!)) {
            toggle_element_visibility(this.panel!);
        }
    }

    do_select_fractal() {
        this.set_visibility(true);
        /* TODO:
         * Populate a list of JS objects ... this is a React Component, I think ... see React "Your First Component" & "Rendering Lists" help. 
         * Action on selection
         */
    }

    noop() { }
}
