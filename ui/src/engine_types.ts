// Type information 
// (c) 2024 Ross Younger

/// Twin of Rust GenericError.
export class GenericError {
    error: string = "";
}

/// Twin of Rust ViewerTileSpec struct. This class is also the userData element of ImageJob.userData.
export class TileSpec {
    serial: number;
    level: number;
    dx: number;
    dy: number;
    width: number;
    height: number;
    max_iter: number;
    algorithm: string;
    colourer: string;
    constructor(serial: number, data: TilePostData, width: number, height: number, algorithm: string, max_iter: number, colourer: string) {
        this.serial = serial; // Always obtain from gSerial.next() !
        this.level = data?.level || 0;
        this.dx = data?.dx || 0;
        this.dy = data?.dy || 0;
        this.width = width;
        this.height = height;
        this.max_iter = max_iter;
        this.algorithm = algorithm;
        this.colourer = colourer;
    }
}

/// Twin of Rust TileResponse struct.
export class TileResponse {
    serial: number;
    rgba_blob: Uint8Array;
    constructor() {
        this.serial = 0;
        this.rgba_blob = new Uint8Array();
    }
}

// Helper class for TileResponse.
// NB that when instantiated by tauri, TileResponse is plain old data without functions. Hence this wrapper.
export class TileResponseHelper {
    tr: TileResponse;
    constructor(tr: TileResponse) {
        this.tr = tr;
    }
    blob(): Uint8ClampedArray {
        return new Uint8ClampedArray(this.tr.rgba_blob);
    }
    image(expected_size: number): ImageData {
        return new ImageData(this.blob(), expected_size, expected_size, { "colorSpace": "srgb" });
    }
    canvas(expected_size: number): HTMLCanvasElement {
        let c = document.createElement("canvas");
        c.width = expected_size;
        c.height = expected_size;
        let ctx = c.getContext("2d");
        ctx?.putImageData(this.image(expected_size), 0, 0);
        return c;
    }
}

/// Twin of Rust TileError struct.
export class TileError {
    serial: number;
    error: string;
    constructor() {
        this.serial = 0;
        this.error = "";
    }
}

/// Class used within OpenSeadragon as a bridge to our TileSpec
export class TilePostData {
    dx: number;
    dy: number;
    level: number;
    constructor(l: number, x: number, y: number) {
        this.dx = x;
        this.dy = y;
        this.level = l;
    }
    toString(): string {
        return `${this.level}/${this.dx}-${this.dy}`;
    }
};

/// Twin of Rust SerializablePoint struct.
/// Named EnginePoint to distinguish from OpenSeadragon.Point
export class EnginePoint {
    re: number = 0.0;
    im: number = 0.0;
    constructor(re: number, im: number) {
        this.re = re;
        this.im = im;
    }
    static fromDict(dict: any) {
        return new EnginePoint(dict.re as number, dict.im as number);
    }
    toString(): string {
        return `{${this.re}, ${this.im}}`;
    }
}

/// Description of a view into the fractal.
/// This could also be the overall fractal dimensions (which we refer to as its _metadata_).
/// Twin of Rust FractalView struct.
export class FractalView {
    origin: EnginePoint = new EnginePoint(0.0, 0.0);
    axes_length: EnginePoint = new EnginePoint(0.0, 0.0);
    end_corner(): EnginePoint {
        return new EnginePoint(this.origin.re + this.axes_length.re, this.origin.im + this.axes_length.im);
    }
    centre(): EnginePoint {
        return new EnginePoint(this.origin.re + 0.5 * this.axes_length.re, this.origin.im + 0.5 * this.axes_length.im);
    }
    constructor(origin?: EnginePoint, axes?: EnginePoint) {
        if (origin !== undefined) {
            this.origin = origin;
        }
        if (axes !== undefined) {
            this.axes_length = axes;
        }
    }
    static fromDict(dict: any) {
        return new FractalView(
            EnginePoint.fromDict(dict.origin),
            EnginePoint.fromDict(dict.axes_length),
        );
    }
    toString(): string {
        return `FV[or=${this.origin},ax=${this.axes_length}]`;
    }
}

/// A complete spec for a fractal plot the user wants drawn.
/// Twin of Rust RenderSpec struct.
export class RenderSpec {
    origin: EnginePoint;
    axes: EnginePoint;
    width: Number;
    height: Number;
    maxiter: Number;
    algorithm: String = "Original";
    colourer: String = "LogRainbow";
    constructor(origin: EnginePoint, axes: EnginePoint, width: Number, height: Number, max_iter: Number) {
        this.origin = origin;
        this.axes = axes;
        this.width = width;
        this.height = height;
        this.maxiter = max_iter;
    }
    set_algorithm(algorithm: string): RenderSpec {
        this.algorithm = algorithm;
        return this;
    }
    set_colourer(colourer: string): RenderSpec {
        this.colourer = colourer;
        return this;
    }
}

/// A representation of a listable item.
/// Twin of rust ListItem struct.
export class ListItem {
    /// Item name
    name: string;
    /// Item description
    description: string;
    constructor(name: string, description: string) {
        this.name = name;
        this.description = description;
    }
}

/// Wrapper for a ListItem, with a key (to keep React happy)
export class ListItemWithKey {
    /// Item ID
    key: string;
    /// Item name
    name: string;
    /// Item description
    description: string;
    constructor(item: ListItem) {
        this.key = item.name;
        this.name = item.name;
        this.description = item.description;
    }
}

export function add_keys_to_list(items: ListItem[]): ListItemWithKey[] {
    return items.map((it: ListItem) => {
        return new ListItemWithKey(it);
    });
}
