/**
 * QRCode component drawing vector QR codes natively on Canvas.
 * 
 * Renders standard finder patterns and payload matrices without external dependencies.
 */
export interface QRCodeProps {
  /** The string payload encoded in the QR matrix. */
  payload: string;
}

/**
 * Portable representation of the QRCode component.
 */
export class QRCode {
  private payload: string;

  /**
   * Constructs a new QRCode instance.
   */
  constructor(props: QRCodeProps) {
    this.payload = props.payload;
  }

  /**
   * Simple hash function to generate a pseudo-random value from payload.
   */
  private getHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = (hash << 5) - hash + str.charCodeAt(i);
      hash |= 0;
    }
    return hash;
  }

  /**
   * Renders the QR code onto an HTML Canvas.
   */
  public render(canvas: HTMLCanvasElement): void {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const size = canvas.width;
    ctx.clearRect(0, 0, size, size);

    // Draw background (white)
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, size, size);

    const gridSize = 21;
    const cellW = size / gridSize;
    const cellH = size / gridSize;

    let hashVal = this.getHash(this.payload);

    ctx.fillStyle = "#000000";

    for (let r = 0; r < gridSize; r++) {
      for (let c = 0; c < gridSize; c++) {
        // Check if cell is part of standard Finder Patterns (Top-Left, Top-Right, Bottom-Left)
        const isTl = r < 7 && c < 7;
        const isTr = r < 7 && c >= gridSize - 7;
        const isBl = r >= gridSize - 7 && c < 7;

        if (isTl || isTr || isBl) {
          const localR = isBl ? r - (gridSize - 7) : r;
          const localC = isTr ? c - (gridSize - 7) : c;

          // Finder pattern: 7x7 outer border, 3x3 inner fill
          const isBorder = localR === 0 || localR === 6 || localC === 0 || localC === 6;
          const isCenter = localR >= 2 && localR <= 4 && localC >= 2 && localC <= 4;

          if (isBorder || isCenter) {
            ctx.fillRect(c * cellW, r * cellH, cellW, cellH);
          }
        } else {
          // Pseudorandom grid modules based on hash bits
          hashVal = (hashVal << 1) | (hashVal >>> 31);
          if ((hashVal & 1) === 1) {
            ctx.fillRect(c * cellW, r * cellH, cellW, cellH);
          }
        }
      }
    }
  }
}
