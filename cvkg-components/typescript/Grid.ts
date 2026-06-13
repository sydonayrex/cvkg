/**
 * Grid component for two-dimensional grid layouts.
 * 
 * Provides structural alignment for dashboard layouts.
 */
export interface GridProps {
  /** Column definitions (e.g. "repeat(auto-fill, minmax(120px, 1fr))"). */
  columns?: string;
  /** Gap space between cells. */
  gap?: number;
}

/**
 * Portable representation of the Grid container.
 */
export class Grid {
  private columns: string;
  private gap: number;
  private children: HTMLElement[] = [];

  /**
   * Constructs a new Grid layout instance.
   */
  constructor(props: GridProps = {}) {
    this.columns = props.columns ?? "repeat(auto-fit, minmax(150px, 1fr))";
    this.gap = props.gap ?? 16;
  }

  /**
   * Adds a child element to the Grid.
   */
  public addChild(child: HTMLElement): this {
    this.children.push(child);
    return this;
  }

  /**
   * Renders the grid layout inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.display = "grid";
    wrapper.style.gridTemplateColumns = this.columns;
    wrapper.style.gap = `${this.gap}px`;
    wrapper.style.width = "100%";

    this.children.forEach(child => {
      wrapper.appendChild(child);
    });

    container.appendChild(wrapper);
  }
}
