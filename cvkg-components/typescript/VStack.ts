/**
 * VStack component for vertical layout container.
 * 
 * Arranges children vertically with specified spacing.
 */
export interface VStackProps {
  /** Pixel spacing between children. */
  spacing?: number;
  /** Alignment of children inside the stack container. */
  alignment?: "start" | "center" | "end" | "stretch";
}

/**
 * Portable representation of the VStack container.
 */
export class VStack {
  private spacing: number;
  private alignment: string;
  private children: HTMLElement[] = [];

  /**
   * Constructs a new VStack instance.
   */
  constructor(props: VStackProps = {}) {
    this.spacing = props.spacing ?? 8;
    this.alignment = props.alignment ?? "center";
  }

  /**
   * Adds a child element to the VStack.
   */
  public addChild(child: HTMLElement): this {
    this.children.push(child);
    return this;
  }

  /**
   * Renders the vertical stack layout inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.display = "flex";
    wrapper.style.flexDirection = "column";
    wrapper.style.gap = `${this.spacing}px`;
    wrapper.style.width = "100%";

    switch (this.alignment) {
      case "start":
        wrapper.style.alignItems = "flex-start";
        break;
      case "end":
        wrapper.style.alignItems = "flex-end";
        break;
      case "stretch":
        wrapper.style.alignItems = "stretch";
        break;
      default:
        wrapper.style.alignItems = "center";
    }

    this.children.forEach(child => {
      wrapper.appendChild(child);
    });

    container.appendChild(wrapper);
  }
}
