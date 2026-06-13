/**
 * ZStack component for overlay layout container.
 * 
 * Layers children on top of each other back-to-front.
 */
export interface ZStackProps {
  /** Alignment of children (e.g. center, start). */
  alignment?: "center" | "top-left" | "bottom-right";
}

/**
 * Portable representation of the ZStack overlay container.
 */
export class ZStack {
  private alignment: string;
  private children: HTMLElement[] = [];

  /**
   * Constructs a new ZStack instance.
   */
  constructor(props: ZStackProps = {}) {
    this.alignment = props.alignment ?? "center";
  }

  /**
   * Adds a child element to the ZStack.
   */
  public addChild(child: HTMLElement): this {
    this.children.push(child);
    return this;
  }

  /**
   * Renders the layered stack layout inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.position = "relative";
    wrapper.style.width = "100%";
    wrapper.style.height = "100%";
    wrapper.style.minHeight = "100px";

    this.children.forEach((child, index) => {
      child.style.position = "absolute";
      child.style.zIndex = `${index + 1}`;
      
      if (this.alignment === "center") {
        child.style.top = "50%";
        child.style.left = "50%";
        child.style.transform = "translate(-50%, -50%)";
      } else if (this.alignment === "top-left") {
        child.style.top = "0";
        child.style.left = "0";
      } else if (this.alignment === "bottom-right") {
        child.style.bottom = "0";
        child.style.right = "0";
      }
      
      wrapper.appendChild(child);
    });

    container.appendChild(wrapper);
  }
}
