// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { createRoot, Root } from 'react-dom/client';
import React, { useContext, useState, useEffect } from 'react';
import { effectModalClickOrEscape } from './modal-react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { ListItem, ListItemWithKey, add_keys_to_list } from './engine_types';
import { DisplayMessageDetail } from './menu';



const SelectionModal = () => {
    const [show, setShow] = useState(false);
    const hide = () => {
        setShow(false);
    };
    const ref = effectModalClickOrEscape(() => {
        hide();
    });

    useEffect(() => {
        listen<DisplayMessageDetail>('select', (event) => {
            let id = event.payload.what;
            switch (id) {
                case "fractal":
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
            List data goes here
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
