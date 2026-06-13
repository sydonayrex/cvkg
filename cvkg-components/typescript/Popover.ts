/**
 * Popover component rendering overlay floating bubbles relative to triggers.
 * 
 * Provides interactive floating containers.
 */
export interface PopoverProps {
  /** Alignment coordinates position context. */
  position?: "top" | "bottom" | "left" | "right";
}

/**
 * Portable representation of the Popover floating bubble container.
 */
export class Popover {
  private position: "top" | "bottom" | "left" | "right";
  private isOpen: boolean = false;
  private popoverContent?: HTMLElement;

  /**
   * Constructs a new Popover instance.
   */
  constructor(props: PopoverProps = {}) {
    this.position = props.position ?? "bottom";
  }

  /**
   * Sets the overlay bubble floating content.
   */
  public setPopoverContent(content: HTMLElement): this {
    this.popoverContent = content;
    return this;
  }

  /**
   * Attaches click triggers to open/close popover.
   */
  public attach(trigger: HTMLElement): void {
    trigger.style.position = "relative";
    trigger.style.cursor = "pointer";

    const bubble = document.createElement("div");
    bubble.style.position = "absolute";
    bubble.style.padding = "8px";
    bubble.style.background = "#1e1e24";
    bubble.style.border = "1px solid #333";
    bubble.style.borderRadius = "4px";
    bubble.style.display = "none";
    bubble.style.zIndex = "500";

    if (this.position === "bottom") {
      bubble.style.top = "100%";
      bubble.style.left = "50%";
      bubble.style.transform = "translateX(-50%) translateY(4px)";
    } else if (this.position === "top") {
      bubble.style.bottom = "100%";
      bubble.style.left = "50%";
      bubble.style.transform = "translateX(-50%) translateY(-4px)";
    }

    if (this.popoverContent) {
      bubble.appendChild(this.popoverContent);
    }
    trigger.appendChild(bubble);

    trigger.addEventListener("click", () => {
      this.isOpen = !this.isOpen;
      bubble.style.display = this.isOpen ? "block" : "none";
    });
  }
}
