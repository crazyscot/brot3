// Type information 
// (c) 2024 Ross Younger

/// Twin of Rust ViewerTileSpec struct. This class is also the userData element of ImageJob.userData.
export class TileSpec {
    // TODO: fractal, colourer
    serial: number;
    level: number;
    dx: number;
    dy: number;
    width: number;
    height: number;
    constructor(serial: number, data: TilePostData, width: number, height: number) {
      this.serial = serial; // Always obtain from gSerial.next() !
      this.level = data?.level || 0;
      this.dx = data?.dx || 0;
      this.dy = data?.dy || 0;
      this.width = width;
      this.height = height;
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
  toString(): string {
    return `{${this.re}, ${this.im}}`;
  }
}
/// Twin of Rust FractalMetadata struct. Also used to hold the viewport current/target position.
export class FractalMetadata {
  origin: EnginePoint = new EnginePoint(0.0, 0.0);
  axes_length: EnginePoint = new EnginePoint(0.0, 0.0);
  end_corner(): EnginePoint {
    return new EnginePoint(this.origin.re + this.axes_length.re, this.origin.im + this.axes_length.im);
  }
  centre(): EnginePoint {
    return new EnginePoint(this.origin.re + 0.5 * this.axes_length.re, this.origin.im + 0.5 * this.axes_length.im);
  }
  constructor(origin: EnginePoint|null = null, axes: EnginePoint|null = null) {
    if (origin !== null) {
      this.origin = origin;
    }
    if (axes !== null) {
      this.axes_length = axes;
    }
  }
}
