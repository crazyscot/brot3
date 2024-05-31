// brot3 fractal/colourer selection
// (c) 2024 Ross Younger

import { createRoot, Root } from 'react-dom/client';
import React, { FC, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Tooltip } from 'react-tooltip';
import { ButtonBase } from '@mui/material';
import { styled } from '@mui/material/styles';

import { ListItem, ListItemWithKey, add_keys_to_list } from './engine_types';
import { DisplayMessageDetail } from './menu';
import { effectModalClickOrEscape } from './modal-react';
import { Viewer } from './viewer'
import './selection_overlay.css'

function description_filter(desc: string): string {
    // Some descriptions contain CLI aliases, which we don't care about here.
    const re = /\(alias.*\)/;
    return desc.replace(re, '');
}

const ImageButton = styled(ButtonBase)(({ theme }) => ({
    position: 'relative',
    height: 150,
    [theme.breakpoints.down('sm')]: {
        width: '100% !important', // Overrides inline-style
        height: 100,
    },
    '&:hover, &.Mui-focusVisible': {
        zIndex: 1,
        '& .MuiImageBackdrop-root': {
            opacity: 0.15,
        },
        '& .MuiImageMarked-root': {
            opacity: 0,
        },
        '& .MuiTypography-root': {
            border: '4px solid currentColor',
        },
    },
}));

const ImageSrc = styled('span')({
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    backgroundSize: 'cover',
    backgroundPosition: 'center 40%',
});

const Image = styled('span')(({ theme }) => ({
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: theme.palette.common.white,
}));

const ImageBackdrop = styled('span')(({ theme }) => ({
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    backgroundColor: theme.palette.common.black,
    opacity: 0.4,
    transition: theme.transitions.create('opacity'),
}));

type DisplayItemProps = {
    name: string,
    description: string,
    hideModal: () => void,
    viewer: Viewer | null,
}

const DisplayItem = (props: DisplayItemProps) => {
    const [imageUrl, setImageUrl] = useState("/openseadragon/images/home_rest.png");

    const BUTTON_WIDTH = 150;
    const doClick = () => {
        props.hideModal();
        props.viewer?.set_algorithm(props.name);
    };

    useEffect(() => {
        const id = setTimeout(() => {
            setImageUrl("/openseadragon/images/zoomout_pressed.png");
        }, 2000);
        return () => clearTimeout(id);
    });
    // TODO: Temporary image URL for now.
    return (
        <ImageButton
            focusRipple
            style={{
                width: BUTTON_WIDTH,
            }}
            data-tooltip-id="list-tooltip"
            data-tooltip-content={description_filter(props.description)}
            onClick={doClick}
        >
            <ImageSrc style={{ backgroundImage: `url(${imageUrl})` }} />
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
    const [show, setShow] = useState(false);
    const [listItems, setListItems] = useState<ListItemWithKey[]>([]);

    const hide = () => {
        setShow(false);
    };
    const ref = effectModalClickOrEscape(() => {
        hide();
    });


    useEffect(() => {
        const unlisten = listen<DisplayMessageDetail>('select', (event) => {
            let id = event.payload.what;
            switch (id) {
                case "fractal":
                    invoke('list_fractals', {}).then((reply) => {
                        let fractals = add_keys_to_list((reply as ListItem[])!);
                        setListItems(fractals);
                    });
                    setShow(true);
                    break;
                default:
                    console.error(`unknown select message detail ${id}`);
            }
        });
        return () => {
            unlisten.then(f => f());
        }
    }, [setListItems, setShow]);

    return <>{show && <div className="react-modal">
        <div className="modal-content" ref={ref}>
            <span className="close" id="close-selector" onClick={hide}>&times;</span>
            <h3>Available Fractals</h3>
            <div id="selection-list">{listItems.map((it) => <DisplayItem {...it} hideModal={hide} viewer={viewer} />)}</div>
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
        this.root.render(
            <SelectionModal viewer={viewer} />
        );
    }

    noop() { }
}