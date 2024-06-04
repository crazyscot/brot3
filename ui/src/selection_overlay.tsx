// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { createRoot, Root } from 'react-dom/client';
import React, { FC, useContext, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Tooltip } from 'react-tooltip';

import { ListItem, ListItemWithKey, TileError, TilePostData, TileResponse, TileResponseHelper, TileSpec, add_keys_to_list } from './engine_types';
import { Image, ImageBackdrop, ImageButton, ImageSrc } from './image_button';
import { DisplayMessageDetail } from './menu';
import { effectModalClickOrEscape } from './modal-react';
import { nextSerial } from './serial_allocator';
import { TILE_SIZE, Viewer } from './viewer'

import './selection_overlay.css'

export const PREVIEW_SIZE = 150;
const PREVIEW_LEVEL = Math.floor(Math.log2(PREVIEW_SIZE));

function description_filter(desc: string): string {
    // Some descriptions contain CLI aliases, which we don't care about here.
    const re = /\(alias.*\)/;
    return desc.replace(re, '');
}

// Database of button images (intended to be data URLs).
// Keys are item names; values are URLs.
type ButtonURLContextType = Map<string, string>;
const ButtonURLContext = React.createContext<ButtonURLContextType>(new Map());

type DisplayItemProps = {
    name: string,
    description: string,
    hideModal: () => void,
    viewer: Viewer | null,
}
const DisplayItem = (props: DisplayItemProps) => {
    const urlMap = useContext(ButtonURLContext);

    const doClick = () => {
        props.hideModal();
        props.viewer?.set_algorithm(props.name);
    };

    return (
        <ImageButton
            focusRipple
            style={{
                width: PREVIEW_SIZE,
            }}
            data-tooltip-id="list-tooltip"
            data-tooltip-content={description_filter(props.description)}
            onClick={doClick}
        >
            <ImageSrc style={{ backgroundImage: `url(${urlMap.get(props.name) || ""})` }} />
            <ImageBackdrop className="MuiImageBackdrop-root" />
            <Image>
                {props.name}
            </Image>
            <Tooltip id="list-tooltip" className="list-tooltip" />
        </ImageButton>
    );
};

interface SelectionModalProps {
    viewer: Viewer,
}

const SelectionModal: FC<SelectionModalProps> = ({ viewer }): JSX.Element => {
    const [show, setShow] = useState(false); // visibility of this panel
    const [listItems, setListItems] = useState<ListItemWithKey[]>([]); // simple list of items we care about
    const [ButtonUrls, setButtonUrls] = useState<ButtonURLContextType>(new Map()); // context state, maps items by name to their URLs
    const outstanding = useRef<Map<number, string>>(new Map()); // Open requests to engine. Maps serial numbers to item names.
    const [rendering, setRendering] = useState(false); // prevents infinite loop re-entrancy

    const hide = () => {
        setShow(false);
    };
    const ref = effectModalClickOrEscape(() => {
        hide();
    });
    const listFractals = () => {
        outstanding.current.clear();
        invoke('list_fractals', {})
            .then((reply) => {
                let fractals = add_keys_to_list((reply as ListItem[])!);
                setListItems(fractals);
            });
        setShow(true);
    };
    useEffect(() => {
        // when listItems is set: set up button URL list.
        let map = new Map<string, string>();
        listItems.forEach((f) => {
            map.set(f.name, "");
        });
        setRendering(false);
        setButtonUrls(map);
    }, [listItems]);
    useEffect(() => {
        if (rendering) return;
        // When ButtonURLs is changed and we're not already rendering: kick off a render loop.
        let specs = listItems.map(async (alg) => {
            let postdata = new TilePostData(PREVIEW_LEVEL, 0, 0);
            let colourer = viewer.get_colourer();
            return new TileSpec(await nextSerial(), postdata, TILE_SIZE, TILE_SIZE, alg.name, 32, colourer);
        });
        setRendering(true); // prevents infinite loop re-entrancy
        specs.forEach(async (s) => {
            let s2 = await s;
            outstanding.current.set(s2.serial, s2.algorithm);
            invoke('start_tile', { spec: s2 })
                .catch((e) => {
                    console.error(e);
                });
        });
    }, [ButtonUrls, rendering]);
    useEffect(() => {
        const unlisten1 = listen<TileResponse>('tile_complete', (event) => {
            let tile = event.payload;
            let requestor = outstanding.current.get(tile.serial);
            if (requestor === undefined) return; // Not for us
            outstanding.current.delete(tile.serial);
            let helper = new TileResponseHelper(tile);
            let canvas = helper.canvas(TILE_SIZE);
            let dataUrl = canvas.toDataURL();
            setButtonUrls((prev) => {
                // GOTCHA: Using an updater function here means we can batch multiple updates.
                // Naively cloning without a closure and calling setButtonUrls(new1) causes the updates to trample each other.
                const new1 = new Map(prev);
                return new1.set(requestor, dataUrl);
            });
        });
        const unlisten2 = listen<TileError>('tile_error', (event) => {
            let err = event.payload;
            outstanding.current.delete(err.serial);
            console.log(`Error in selection render job ${err.serial}: ${err.error}`);
        });
        return () => {
            unlisten1
                .then(_ => unlisten2)
                .then(f => f());
        }
    }, []);

    useEffect(() => {
        const unlisten = listen<DisplayMessageDetail>('select', (event) => {
            let id = event.payload.what;
            switch (id) {
                case "fractal":
                    listFractals();
                    break;
                default:
                    console.error(`unknown select message detail ${id}`);
            }
        });
        return () => {
            unlisten.then(f => f());
        }
    }, []);

    return <ButtonURLContext.Provider value={ButtonUrls}>
        {show && <div className="react-modal">
            <div className="modal-content" ref={ref}>
                <span className="close" id="close-selector" onClick={hide}>&times;</span>
                <h3>Available Fractals</h3>
                <div id="selection-list">{listItems.map((it) => <DisplayItem {...it} hideModal={hide} viewer={viewer} />)}</div>
            </div>
        </div>}
    </ButtonURLContext.Provider>
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
        this.root.render(
            <SelectionModal viewer={viewer} />
        );
    }

    noop() { }
}