// brot3 heads up display
// (c) 2024 Ross Younger

import { ClickEventListener, element_is_displayed, toggle_tr_visibility } from './dom_util'
import { EnginePoint, FractalView } from './engine_types'

// Formatting helpers

/// Formats a float with a given number of significant figures (decimal precision).
/// @positive@ is prepended to numbers >= 0.0.
/// @precision@ is the required number of significant figures.
function format_float_with_precision(positive: string, n: number, precision: number): string {
    let strnum;
    if (precision === undefined) {
        strnum = `${n}`;
    } else {
        if (n < 1e-3) {
            strnum = `${n.toExponential(precision)}`;
        } else {
            strnum = `${n.toPrecision(precision)}`;
        }
    }
    if (n >= 0.0)
        return `${positive}${strnum}`;
    return `${strnum}`;
}

/// Formats a float with a given (fixed) number of decimal places.
/// @positive@ is prepended to numbers >= 0.0.
function format_float_fixed(positive: string, n: number, decimal_places: number): string {
    let strnum = `${n.toFixed(decimal_places)}`;
    if (n >= 0.0)
        return `${positive}${strnum}`;
    return `${strnum}`;
}

/// Computes the decimal precision (number of significant figures) required for a given canvas size.
function axes_precision_for_canvas(canvas_height: number, canvas_width: number): number {
    // Rationale: If a change in axes would move us <1 pixel it has no visible effect.
    return Math.ceil(Math.log10(Math.max(canvas_height, canvas_width)));
}
/// Computes the number of decimal places required for a given canvas and axes size.
function decimal_places_for_axes(canvas_height: number, canvas_width: number, axes_length: EnginePoint): number {
    // Rationale: If a change in position would move us <1 pixel it has no visible effect.
    let pixel_size = new EnginePoint(axes_length.re / canvas_width, axes_length.im / canvas_height);
    return Math.ceil(-Math.log10(Math.max(pixel_size.re, pixel_size.im)));
}

export class HeadsUpDisplay {
    static readonly html: string = `
    <div class="info" id="info-panel">
    <form id="form_go_to_position">
      <table>
        <tr class="position-display">
          <td rowspan="4"><span class="close" id="close-hud">&times;</span></td>
        </tr>
        <tr class="position-display hidden" id="show-origin">
          <th>Origin:</th>
          <td><span id="originReal"></span></td>
          <td><span id="originImag"></span> i&nbsp;</td>
        </tr>
        <tr class="position-display" id="show-centre">
          <th>Centre:</th>
          <td><span id="centreReal"></span></td>
          <td><span id="centreImag"></span> i&nbsp;</td>
        </tr>
        <tr class="position-display">
          <th>Axes:</th>
          <td><span id="axesReal"></span> re,</td>
          <td><span id="axesImag"></span> im</td>
        </tr>
        <tr class="position-display">
          <th>Zoom:</th>
          <td id="zoom"></td>
        </tr>
        <!--------------------------------------------->
        <tr class="position-entry">
          <td rowspan="5"><span class="close" id="close-entry">&times;</span></td>
        </tr>
        <tr class="position-entry hidden" id="enter-origin">
          <th>Origin:</th>
          <td><input type="text" id="enter_originReal" /></td>
          <td>+ <input type="text" id="enter_originImag" /> i</td>
          <td colspan="3"></td>
        </tr>
        <tr class="position-entry" id="enter-centre">
          <th>Centre:</th>
          <td><input type="text" id="enter_centreReal" /></td>
          <td>+ <input type="text" id="enter_centreImag" /> i</td>
          <td colspan="3"></td>
        </tr>
        <tr class="position-entry">
          <th>Axis:</th>
          <td colspan="2"><input type="text" id="enter_axesReal" /> real</td>
          <td colspan="2"></td>
        </tr>
        <tr class="position-entry">
          <td></td>
          <td><button type="submit" id="action_go_to_position">Go</button></td>
          <td><button type="button" id="action_copy_current_position">Copy Current</button></td>
          <td colspan="2"></td>
        </tr>
        <tr class="position-entry">
          <td colspan="6" id="position-error-text"></td>
        </tr>
      </table>
    </form>
  </div>
`;

    zoom: Element | null;
    originReal: Element | null;
    originImag: Element | null;
    centreReal: Element | null;
    centreImag: Element | null;
    axesReal: Element | null;
    axesImag: Element | null;

    hud_closer: ClickEventListener;
    position_entry_closer: ClickEventListener;

    constructor(doc: Document) {
        let self = this; // For closures

        var panel = doc.querySelectorAll('#info-panel')[0];
        this.zoom = panel.querySelectorAll('#zoom')[0];
        this.originReal = panel.querySelectorAll('#originReal')[0];
        this.originImag = panel.querySelectorAll('#originImag')[0];
        this.centreReal = panel.querySelectorAll('#centreReal')[0];
        this.centreImag = panel.querySelectorAll('#centreImag')[0];
        this.axesReal = panel.querySelectorAll('#axesReal')[0];
        this.axesImag = panel.querySelectorAll('#axesImag')[0];

        // Hide the position entry rows by default
        this.position_entry_rows().forEach(e => { if (element_is_displayed(e)) toggle_tr_visibility(e); });
        this.hud_closer = new ClickEventListener(
            document.getElementById("close-hud")!,
            function (_event: Event) {
                self.toggle_visibility();
            }
        );
        this.position_entry_closer = new ClickEventListener(
            document.getElementById("close-entry")!,
            function (_event: Event) {
                self.toggle_position_entry_panel();
            }
        );
    }

    update(zoom: number, origin: EnginePoint, centre: EnginePoint, axes: EnginePoint, canvas_width: number, canvas_height: number) {
        let axes_precision = axes_precision_for_canvas(canvas_height, canvas_width);
        let position_dp = decimal_places_for_axes(canvas_height, canvas_width, axes);
        this.zoom!.innerHTML = `${zoom.toPrecision(axes_precision)} &times;`;
        this.axesReal!.innerHTML = format_float_with_precision("&nbsp;", axes.re, axes_precision);
        this.axesImag!.innerHTML = format_float_with_precision("&nbsp;", axes.im, axes_precision);
        this.centreReal!.innerHTML = format_float_fixed("&nbsp;", centre.re, position_dp);
        this.centreImag!.innerHTML = format_float_fixed("+", centre.im, position_dp);
        this.originReal!.innerHTML = format_float_fixed("&nbsp;", origin.re, position_dp);
        this.originImag!.innerHTML = format_float_fixed("+", origin.im, position_dp);
    }

    toggle_visibility() {
        let elements = Array.from(document.querySelectorAll('tr.position-display'), e => e as HTMLElement);
        elements.forEach(e => toggle_tr_visibility(e));
    }

    // Copy the current position into the Go To Position form
    set_go_to_position(pos: FractalView, canvas_width: number, canvas_height: number) {
        let axes_precision = axes_precision_for_canvas(canvas_width, canvas_height);
        let f = document.getElementById("enter_axesReal")! as HTMLInputElement;
        f.value = format_float_with_precision("", pos.axes_length.re, axes_precision);
        let position_dp = decimal_places_for_axes(canvas_height, canvas_width, pos.axes_length);

        if (this.origin_is_currently_visible()) {
            f = document.getElementById("enter_originReal")! as HTMLInputElement;
            f.value = format_float_fixed("", pos.origin.re, position_dp);
            f = document.getElementById("enter_originImag")! as HTMLInputElement;
            f.value = format_float_fixed("", pos.origin.im, position_dp);
        } else {
            f = document.getElementById("enter_centreReal")! as HTMLInputElement;
            f.value = format_float_fixed("", pos.centre().re, position_dp);
            f = document.getElementById("enter_centreImag")! as HTMLInputElement;
            f.value = format_float_fixed("", pos.centre().im, position_dp);
        }

        // clear out: Axes Im, Zoom
        ["axesImag", "zoom"].forEach(f => {
            let field = document.getElementById("enter_" + f)! as HTMLInputElement;
            if (field !== null) {
                field.value = "";
            }
        });
    }

    toggle_origin_centre() {
        if (this.origin_is_currently_visible()) {
            document.getElementById("show-origin")?.classList.add("hidden");
            document.getElementById("show-centre")?.classList.remove("hidden");
            document.getElementById("enter-origin")?.classList.add("hidden");
            document.getElementById("enter-centre")?.classList.remove("hidden");
            // Clear out fields we just hid
            let field = document.getElementById("enter_originReal") as HTMLInputElement;
            field.value = "";
            field = document.getElementById("enter_originImag") as HTMLInputElement;
            field.value = "";
        } else {
            document.getElementById("show-origin")?.classList.remove("hidden");
            document.getElementById("show-centre")?.classList.add("hidden");
            document.getElementById("enter-origin")?.classList.remove("hidden");
            document.getElementById("enter-centre")?.classList.add("hidden");
            // Clear out fields we just hid
            let field = document.getElementById("enter_centreReal") as HTMLInputElement;
            field.value = "";
            field = document.getElementById("enter_centreImag") as HTMLInputElement;
            field.value = "";
        }
    }

    private position_entry_rows(): HTMLElement[] {
        return Array.from(document.querySelectorAll('tr.position-entry'), e => e as HTMLElement);
    }

    toggle_position_entry_panel() {
        let visible = false;
        this.position_entry_rows().forEach(e => visible = toggle_tr_visibility(e));
        if (visible) {
            let element = undefined;
            if (this.origin_is_currently_visible()) {
                element = document.getElementById(`enter_originReal`) as HTMLInputElement;
            } else {
                element = document.getElementById(`enter_centreReal`) as HTMLInputElement;
            }
            element!.focus();
            element!.select();
        }
    }

    parse_entered_position(): UserDestination {
        let result = new UserDestination();
        let errors = new Array<string>;
        let k: keyof UserDestination;
        for (k in result) {
            let fieldId = "#enter_" + k;
            let fieldElement = document.querySelector<HTMLInputElement>(fieldId);
            if (fieldElement === null) {
                // quietly ignore it
                continue;
            }
            let value = parseFloat(fieldElement.value);
            // this results in NaN if a field is empty; that's OK as not all are mandatory. Viewer will figure it out.
            result[k] = value;
        };
        if (errors.length !== 0) {
            let message = `Form data error: ${errors.join(", ")}`;
            throw new Error(message);
        }
        return result;
    }

    origin_is_currently_visible(): boolean {
        let originDisplay = document.getElementById("show-origin");
        return !originDisplay?.classList.contains("hidden");
    }
}

// Where does the user want to go today?
// Not all these fields will necessarily be visible at once.
export class UserDestination {
    zoom: number = NaN;
    centreReal: number = NaN;
    centreImag: number = NaN;
    originReal: number = NaN;
    originImag: number = NaN;
    axesReal: number = NaN;
    axesImag: number = NaN;
}
