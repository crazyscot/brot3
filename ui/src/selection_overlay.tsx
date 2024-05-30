// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { createRoot, Root } from 'react-dom/client';
import React, { useState, useEffect } from 'react';
import { effectModalClickOrEscape } from './modal-react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { ListItem, ListItemWithKey, add_keys_to_list } from './engine_types';
import { DisplayMessageDetail } from './menu';

const DisplayItem = ({ name = "", description = "", key = 0 }) => {
    return (
        <li
            className="listItem"
            role="button"
            key={key}
        >
            <b className="listItemName">{name}</b>
        </li>
    );
};

const SelectionModal = () => {
    const [show, setShow] = useState(false);
    const hide = () => {
        setShow(false);
    };
    const ref = effectModalClickOrEscape(() => {
        hide();
    });

    const [listData, setListData] = useState<ListItemWithKey[]>([]);

    useEffect(() => {
        listen<DisplayMessageDetail>('select', (event) => {
            let id = event.payload.what;
            switch (id) {
                case "fractal":
                    invoke('list_fractals', {}).then((reply) => {
                        let fractals = add_keys_to_list((reply as ListItem[])!);
                        setListData(fractals);
                    });
                    setShow(true);
                    break;
                default:
                    console.error(`unknown select message detail ${id}`);
            }
        });
    }, []);

    return <>{show && <div className="reactmodal">
        <div className="modal-content" ref={ref}>
            <span className="close" id="close-selector" onClick={hide}>&times;</span>
            <h3>Select Fractal</h3>
            <ul>{listData.map(it => DisplayItem(it))}</ul>
        </div>
    </div>}</>
};

export class SelectionOverlay {
    // Our base html, which is a React root
    static readonly html: string = `<div id="selector-panel"></div>`;

    panel: HTMLElement | null;
    readonly root: Root;

    constructor(doc: Document) {
        //let self = this; // For closures
        this.panel = doc.querySelectorAll('#selector-panel')[0] as HTMLElement;

        this.root = createRoot(this.panel);
        this.root.render(<SelectionModal />);
    }

    noop() { }
}

// TODO draw as boxes.. title, mouseOver text is description.
// TODO word wrap the boxes
// TODO make it scrollable when the window is small
// TODO previews
// TODO actions
// TODO what about the CLI aliases?
