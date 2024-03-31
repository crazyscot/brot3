// utility code affecting the Document Object Model
// (c) 2024 Ross Younger

export function tr_is_visible(e: HTMLElement): boolean {
    return e.style.display !== "none";
}

// Toggles visibility of a TR element, returning true iff it is now visible.
export function toggle_tr_visibility(e: HTMLElement): boolean {
    if (e.style.display === "none") {
        e.style.display = "table-row";
        return true;
    } else {
        e.style.display = "none";
        return false;
    }
}

export class ClickEventListener implements EventListenerObject {
    clickable: Element;
    action: Function;
    constructor(target: Element, action: Function) {
        this.clickable = target;
        this.clickable.addEventListener("click", this);
        this.action = action;
    }
    handleEvent(event: Event): void | Promise<void> {
        if (event.type === "click") {
            this.action(event);
        }
    }
}
