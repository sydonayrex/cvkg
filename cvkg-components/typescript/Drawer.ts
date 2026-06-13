/**
 * Drawer component representing sliding overlay side panels.
 * 
 * Provides layout drawers side panels.
 */
export interface DrawerProps {
  /** Alignment slide position target. */
  position?: "left" | "right";
  /** Whether the slide panel is initially visible. */
  isOpen?: boolean;
}

/**
 * Portable representation of the Drawer side panel.
 */
export class Drawer {
  private position: "left" | "right";
  private isOpen: boolean;
  private contentElement?: HTMLElement;

  /**
   * Constructs a new Drawer instance.
   */
  constructor(props: DrawerProps = {}) {
    this.position = props.position ?? "right";
    this.isOpen = props.isOpen ?? false;
  }

  /**
   * Sets the content body inside the slide panel.
   */
  public setContent(content: HTMLElement): this {
    this.contentElement = content;
    return this;
  }

  /**
   * Toggles the open state.
   */
  public setOpen(open: boolean, container: HTMLElement): void {
    this.isOpen = open;
    this.render(container);
  }

  /**
   * Renders the drawer slide overlay inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    if (!this.isOpen) return;

    const overlay = document.createElement("div");
    overlay.style.position = "fixed";
    overlay.style.top = "0";
    overlay.style.bottom = "0";
    overlay.style.width = "300px";
    overlay.style.background = "#18181c";
    overlay.style.borderLeft = "1px solid #333";
    overlay.style.borderRight = "1px solid #333";
    overlay.style.boxShadow = "0 0 10px rgba(0,0,0,0.5)";
    overlay.style.zIndex = "1000";
    overlay.style.padding = "16px";

    if (this.position === "left") {
      overlay.style.left = "0";
    } else {
      overlay.style.right = "0";
    }

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "Close Drawer";
    closeBtn.style.width = "100%";
    closeBtn.style.padding = "8px";
    closeBtn.style.marginBottom = "12px";
    closeBtn.addEventListener("click", () => {
      this.isOpen = false;
      overlay.remove();
    });
    overlay.appendChild(closeBtn);

    if (this.contentElement) {
      overlay.appendChild(this.contentElement);
    }

    container.appendChild(overlay);
  }
}
