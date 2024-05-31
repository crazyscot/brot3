// Modal dialog in React
// (c) 2024 Ross Younger

import { useEffect, useRef } from 'react';

// Composite effect for modals: When there's a click outside of some element, or Escape is pressed, call a given callback.
export const effectModalClickOrEscape = (callback: () => void) => {
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent | TouchEvent) => {
            if (ref.current && !ref.current.contains(event.target as Node)) {
                callback();
            }
        };
        document.addEventListener('mouseup', handleClickOutside);
        document.addEventListener('touchend', handleClickOutside);
        return () => {
            document.removeEventListener('mouseup', handleClickOutside);
            document.removeEventListener('touchend', handleClickOutside);
        };
    }, [callback, ref]);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (ref.current && event.key === 'Escape') {
                callback();
            }
        };
        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [callback, ref]);
    return ref;
};
