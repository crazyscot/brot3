// brot3 about dialog
// (c) 2024 Ross Younger

import './modal.css'

export class About implements EventListenerObject {
    static readonly html: string = `
<div id="aboutModal" class="modal">
  <!-- Modal content -->
  <div class="modal-content">
    <span class="close">&times;</span>
    <p>brot3 &copy; 2024 Ross Younger</p>
    <p>This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.</p>

    <p>This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.</p>

    <p>You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see https://www.gnu.org/licenses/</p>

  </div>
</div>
    `;

    modal: HTMLElement;
    close_x: Element;

    constructor(modal: HTMLElement) {
        //let self = this;
        this.modal = modal;

        var span: Element = modal.getElementsByClassName("close")[0];
        this.close_x = span;
        // caution: memory leak if called repeatedly; don't do that, then

        // click the X to close
        span.addEventListener("click", this);
        // click outside to close
        window.addEventListener("click", this);
    }

    handleEvent(event: Event): void | Promise<void> {
        if (event.type === "click") {
            this.maybe_hide(event);
        }
    }

    show() {
        this.modal!.style.display = "block";
    }

    hide() {
        this.modal!.style.display = "none";
    }
    maybe_hide(event: Event) {
        if (event.target === this.modal || event.target === this.close_x) {
            this.hide();
        }
    }
}