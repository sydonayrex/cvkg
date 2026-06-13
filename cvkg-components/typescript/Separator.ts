/**
 * Separator component rendering line dividers.
 * 
 * Separates layout blocks visually.
 */
export interface SeparatorProps {
  /** Alignment orientation of the separator line. */
  orientation?: "horizontal" | "vertical";
  /** Optional hex or css color code. */
  color?: string;
  /** Thickness of the separator line. */
  thickness?: number;
}

/**
 * Portable representation of the Separator line.
 */
export class Separator {
  private orientation: "horizontal" | "vertical";
  private color: string;
  private thickness: number;

  /**
   * Constructs a new Separator instance.
   */
  constructor(props: SeparatorProps = {}) {
    this.orientation = props.orientation ?? "horizontal";
    this.color = props.color ?? "#33333d";
    this.thickness = props.thickness ?? 1;
  }

  /**
   * Renders the separator element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const line = document.createElement("div");
    line.style.background = this.color;

    if (this.orientation === "horizontal") {
      line.style.width = "100%";
      line.style.height = `${this.thickness}px`;
      line.style.margin = "8px 0";
    } else {
      line.style.width = `${this.thickness}px`;
      line.style.height = "100%";
      line.style.minHeight = "24px";
      line.style.margin = "0 8px";
      line.style.display = "inline-block";
      line.style.verticalAlign = "middle";
    }

    container.appendChild(line);
  }
}
