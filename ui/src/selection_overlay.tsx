// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { createRoot, Root } from 'react-dom/client';
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Tooltip } from 'react-tooltip';
import { Button } from '@mui/material';

import { ListItem, ListItemWithKey, add_keys_to_list } from './engine_types';
import { DisplayMessageDetail } from './menu';
import { effectModalClickOrEscape } from './modal-react';
import { Viewer } from './viewer'
import './selection_overlay.css'

const DisplayItem = ({ name = "", description = "", key = 0 }, hideModal = () => { }) => {
    const doClick = () => {
        console.log(`select ${name}`);
        hideModal();
        // change fractal - do we need a Context for this?
    };
    return (
        <span key={key}>
            <Button
                variant="outlined"
                data-tooltip-id="list-tooltip"
                data-tooltip-content={description}
                onClick={doClick}
            >
                {name}
            </Button>
            <Tooltip id="list-tooltip" className="list-tooltip" />
        </span>
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

    return <>{show && <div className="react-modal">
        <div className="modal-content" ref={ref}>
            <span className="close" id="close-selector" onClick={hide}>&times;</span>
            <h3>Available Fractals</h3>
            <div id="selection-list">{listData.map(it => DisplayItem(it, hide))}</div>
        </div>
    </div>}</>
};

export class SelectionOverlay {
    // Our base html, which is a React root
    static readonly html: string = `<div id="selector-panel"></div>`;

    panel: HTMLElement | null;
    readonly root: Root;
    readonly viewer: Viewer;

    constructor(doc: Document, viewer: Viewer) {
        //let self = this; // For closures
        this.panel = doc.querySelectorAll('#selector-panel')[0] as HTMLElement;
        this.viewer = viewer;

        this.root = createRoot(this.panel);
        this.root.render(<SelectionModal />);
    }

    noop() { }
}

// TODO make it scrollable when the window is small
// TODO previews
// TODO actions
// TODO what about the CLI aliases?
