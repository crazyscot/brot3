// Size-to-save interactor
// (c) 2024 Ross Younger

import { ClickEventListener, element_is_displayed, toggle_element_visibility } from "./dom_util";
import { invoke } from '@tauri-apps/api'

import { FractalView, RenderSpec } from './engine_types'
import { Viewer } from "./viewer";

export class SaveSizeBox {
    static readonly html: string = `
    <div class="user-form" id="save-size-block" style="display: none;">
    <form id="save_size_form">
        <table>
            <tr>
            <td><span class="close" id="close-save-size">&times;</span></td>
            <td>
                <strong>Save Image</strong>
                width: <input type="text" id="enter_save_width" size="5" />
                height: <input type="text" id="enter_save_height" size="5" />
                <button type="submit" id="action_save_size">Save</button>
            </td>
            </tr>
        </table>
    </form>
    </div>
    `;

    doc: Document;
    viewer: Viewer;

    our_div: HTMLElement;
    closer: ClickEventListener;
    heightField: HTMLInputElement;
    widthField: HTMLInputElement;

    constructor(doc: Document, viewer: Viewer) {
        let self = this; // For closures
        this.doc = doc;
        this.viewer = viewer;

        this.our_div = doc.querySelectorAll('#save-size-block')![0] as HTMLElement;
        this.closer = new ClickEventListener(
            this.our_div.querySelectorAll("#close-save-size")![0],
            function (_event: Event) {
                toggle_element_visibility(self.our_div);
            }
        );
        this.widthField = this.doc.getElementById(`enter_save_width`) as HTMLInputElement;
        this.heightField = this.doc.getElementById(`enter_save_height`) as HTMLInputElement;

        // Bind form actions
        doc.getElementById("save_size_form")!.onsubmit = function (e) {
            e.preventDefault();
            self.save_at_entered_size();
        }

        window.addEventListener('resize', function (_) {
            self.populate_from_current_size();
        });
    }

    show() {
        if (!element_is_displayed(this.our_div)) {
            toggle_element_visibility(this.our_div);
        }
        this.populate_from_current_size();
        this.widthField!.focus();
        this.widthField!.select();
    }

    populate_from_current_size() {
        this.widthField.value = window.innerWidth.toString();
        this.heightField.value = window.innerHeight.toString();
    }

    save_at_one_size_or_other(width: number, height: number) {
        // If the input box is visible, that takes priority. Otherwise, fall back to what was passed in (usually the current window size).
        if (!element_is_displayed(this.our_div)) {
            this.do_save(width, height);
        } else {
            this.save_at_entered_size();
        }
    }
    save_at_entered_size() {
        let w = Number.parseInt(this.widthField.value);
        let h = Number.parseInt(this.heightField.value);
        if (Number.isFinite(w) && Number.isFinite(h)) {
            this.do_save(w, h);
        } else {
            console.warn(`failed to parse save size ${w} x ${h}`);
        }
    }
    do_save(width: number, height: number) {
        let position = this.viewer.get_position();
        let view = new FractalView(position.origin, position.axes_length);
        let fixed = this.fix_aspect_ratio(view, width, height);
        let max_iter = this.viewer.get_max_iter();
        invoke('save_image_workflow', {
            spec: new RenderSpec(fixed.origin, fixed.axes_length, width, height, max_iter)
                .set_algorithm(this.viewer.get_algorithm())
                .set_colourer(this.viewer.get_colourer())
        });
    }

    // Auto-adjust the plot axes to match the given pixel size, because that was what you wanted
    fix_aspect_ratio(plot: FractalView, width: number, height: number): FractalView {
        // this is basically a port of auto_adjust_aspect_ratio() from rust
        let result = plot;
        let axes_aspect = plot.axes_length.re / plot.axes_length.im;
        let pixels_aspect = width / height;
        let meta_ratio = pixels_aspect / axes_aspect;
        let centre = plot.centre();
        if (axes_aspect < pixels_aspect) {
            // The requested pixel dimensions are too narrow.
            // Grow the plot in Real, maintaining its centre.
            result.axes_length.re *= meta_ratio;
            // Recompute origin to keep the same centre
            result.origin.re = centre.re - 0.5 * plot.axes_length.re;
            result.origin.im = centre.im - 0.5 * plot.axes_length.im;
            console.log('fixed aspect ratio', result);
        } else if (axes_aspect > pixels_aspect) {
            // The requested pixel dimensions are too tall.
            // Grow the plot in Imaginary, maintaining its centre.
            result.axes_length.im /= meta_ratio;
            // Recompute origin to keep the same centre
            result.origin.re = centre.re - 0.5 * plot.axes_length.re;
            result.origin.im = centre.im - 0.5 * plot.axes_length.im;
            console.log('fixed aspect ratio', result);
        } else {
            // nothing to do
        }
        return result;
    }
}