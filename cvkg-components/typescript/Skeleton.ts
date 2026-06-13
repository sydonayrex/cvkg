/**
 * Skeleton loader component for shimmer loading indicators.
 * 
 * Prevents layout shifts while contents resolve.
 */
export interface SkeletonProps {
  /** Optional visual block width. */
  width?: string;
  /** Optional visual block height. */
  height?: string;
  /** Circular border rounding. */
  circle?: boolean;
}

/**
 * Portable representation of the Skeleton loading loader.
 */
export class Skeleton {
  private width: string;
  private height: string;
  private circle: boolean;

  /**
   * Constructs a new Skeleton instance.
   */
  constructor(props: SkeletonProps = {}) {
    this.width = props.width ?? "100%";
    this.height = props.height ?? "16px";
    this.circle = props.circle ?? false;
  }

  /**
   * Renders the skeleton box element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const block = document.createElement("div");
    block.style.width = this.width;
    block.style.height = this.height;
    block.style.borderRadius = this.circle ? "50%" : "4px";
    block.style.background = "linear-gradient(90deg, #1e1e24 25%, #2d2d34 50%, #1e1e24 75%)";
    block.style.backgroundSize = "200% 100%";
    block.style.animation = "shimmer 1.5s infinite linear";

    // Inject temporary keyframes if not already present
    if (!document.getElementById("skeleton-animation-style")) {
      const style = document.createElement("style");
      style.id = "skeleton-animation-style";
      style.textContent = `
        @keyframes shimmer {
          0% { background-position: 200% 0; }
          100% { background-position: -200% 0; }
        }
      `;
      document.head.appendChild(style);
    }

    container.appendChild(block);
  }
}
