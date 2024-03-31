// brot3 heads up display
// (c) 2024 Ross Younger

import { ClickEventListener, tr_is_visible, toggle_tr_visibility } from './dom_util'
import { EnginePoint, FractalMetadata } from './engine_types'

function maybe_leading(symbol: string, n: number): string {
    if (n >= 0.0)
        return `${symbol}${n}`;
    return `${n}`;
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
        this.position_entry_rows().forEach(e => { if (tr_is_visible(e)) toggle_tr_visibility(e); });
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

    update(zoom: number, origin: EnginePoint, centre: EnginePoint, axes: EnginePoint) {
        this.zoom!.innerHTML = `${zoom.toPrecision(4)} &times;`;
        this.axesReal!.innerHTML = maybe_leading("&nbsp;", axes.re);
        this.axesImag!.innerHTML = maybe_leading("&nbsp;", axes.im);
        this.centreReal!.innerHTML = maybe_leading("&nbsp;", centre.re);
        this.centreImag!.innerHTML = maybe_leading("+", centre.im);
        this.originReal!.innerHTML = maybe_leading("&nbsp;", origin.re);
        this.originImag!.innerHTML = maybe_leading("+", origin.im);
    }

    toggle_visibility() {
        let elements = Array.from(document.querySelectorAll('tr.position-display'), e => e as HTMLElement);
        elements.forEach(e => toggle_tr_visibility(e));
    }

    // Copy the current position into the Go To Position form
    set_go_to_position(pos: FractalMetadata) {
        let f = document.getElementById("enter_axesReal")! as HTMLInputElement;
        f.value = pos.axes_length.re.toString();

        if (this.origin_is_currently_visible()) {
            f = document.getElementById("enter_originReal")! as HTMLInputElement;
            f.value = pos.origin.re.toString();
            f = document.getElementById("enter_originImag")! as HTMLInputElement;
            f.value = pos.origin.im.toString();
        } else {
            f = document.getElementById("enter_centreReal")! as HTMLInputElement;
            f.value = pos.centre().re.toString();
            f = document.getElementById("enter_centreImag")! as HTMLInputElement;
            f.value = pos.centre().im.toString();
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

    private position_entry_rows() : HTMLElement[] {
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

    parse_entered_position() : Map<string, number> {
        let result = new Map<string, number>();
        let errors = new Array<string>;
        for (let f of position_entry_fields) {
            let fieldId = "#enter_" + f;
            let fieldElement = document.querySelector<HTMLInputElement>(fieldId);
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
  
    origin_is_currently_visible(): boolean {
        let originDisplay = document.getElementById("show-origin");
        return !originDisplay?.classList.contains("hidden");
    }
}
