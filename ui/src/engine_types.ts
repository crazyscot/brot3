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
