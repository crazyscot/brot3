// Iteration limit interactor
// (c) 2024 Ross Younger

import { ClickEventListener, element_is_displayed, toggle_element_visibility } from "./dom_util";
//import { invoke } from '@tauri-apps/api'

//import { FractalView, RenderSpec } from './engine_types'
import { Viewer } from "./viewer";

export class IterationLimitBox {
    static readonly html: string = `
    <div class="user-form" id="max-iter-block" style="display: none;">
    <form id="max_iter_form">
        <table><tr>
            <td><span class="close" id="close-max-iter">&times;</span></td>
            <td>
                <strong>Max Iterations</strong>
                <input type="text" id="enter_max_iter" size="7" />
                <button type="submit" id="action_set_max_iter">Redraw</button>
            </td>
        </tr></table>
    </form>
    </div>
    `;

    doc: Document;
    viewer: Viewer;

    our_div: HTMLElement;
    closer: ClickEventListener;
    maxIterField: HTMLInputElement;

    constructor(doc: Document, viewer: Viewer) {
        let self = this; // For closures
        this.doc = doc;
        this.viewer = viewer;

        this.our_div = doc.querySelectorAll('#max-iter-block')![0] as HTMLElement;
        this.closer = new ClickEventListener(
            this.our_div.querySelectorAll("#close-max-iter")![0],
            function (_event: Event) {
                toggle_element_visibility(self.our_div);
            }
        );
        this.maxIterField = this.doc.getElementById(`enter_max_iter`) as HTMLInputElement;

        // Bind form actions
        doc.getElementById("max_iter_form")!.onsubmit = function (e) {
            e.preventDefault();
            self.apply();
        }
    }

    show() {
        if (!element_is_displayed(this.our_div)) {
            toggle_element_visibility(this.our_div);
        }
        // TODO: Populate from current value
        //this.maxIterField.value = ...;
        this.maxIterField!.focus();
        this.maxIterField!.select();
    }

    apply() {
        let new_max = Number.parseInt(this.maxIterField.value);
        if (Number.isFinite(new_max)) {
            console.log(`TODO: apply new max_iter ${new_max}`);
        } else {
            console.warn(`failed to parse max_iter ${new_max}`);
        }
    }
}