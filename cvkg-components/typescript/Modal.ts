/**
 * Modal component for modal dialog boxes.
 * 
 * Provides structural overlays blocks.
 */
export interface ModalProps {
  /** Text heading header. */
  title: string;
  /** Whether the modal dialog box is initially visible. */
  isOpen?: boolean;
  /** Callback triggered when modal closes. */
  onClose?: () => void;
}

/**
 * Portable representation of the Modal overlay dialog.
 */
export class Modal {
  private title: string;
  private isOpen: boolean;
  private onClose?: () => void;
  private contentElement?: HTMLElement;

  /**
   * Constructs a new Modal instance.
   */
  constructor(props: ModalProps) {
    this.title = props.title;
    this.isOpen = props.isOpen ?? false;
    this.onClose = props.onClose;
  }

  /**
   * Sets the content body inside the modal dialog box.
   */
  public setContent(content: HTMLElement): this {
    this.contentElement = content;
    return this;
  }

  /**
   * Toggle the open state.
   */
  public setOpen(open: boolean, container: HTMLElement): void {
    this.isOpen = open;
    this.render(container);
  }

  /**
   * Renders the modal backdrop and overlay dialog inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    if (!this.isOpen) return;

    const backdrop = document.createElement("div");
    backdrop.style.position = "fixed";
    backdrop.style.top = "0";
    backdrop.style.left = "0";
    backdrop.style.width = "100%";
    backdrop.style.height = "100%";
    backdrop.style.background = "rgba(0,0,0,0.6)";
    backdrop.style.display = "flex";
    backdrop.style.alignItems = "center";
    backdrop.style.justifyContent = "center";
    backdrop.style.zIndex = "1000";

    const dialog = document.createElement("div");
    dialog.style.background = "#1e1e24";
    dialog.style.border = "1px solid #333";
    dialog.style.borderRadius = "8px";
    dialog.style.width = "400px";
    dialog.style.padding = "16px";
    dialog.style.display = "flex";
    dialog.style.flexDirection = "column";
    dialog.style.gap = "12px";

    const header = document.createElement("div");
    header.style.display = "flex";
    header.style.justifyContent = "between";
    header.style.alignItems = "center";
    
    const h3 = document.createElement("h3");
    h3.textContent = this.title;
    h3.style.color = "#fff";
    h3.style.margin = "0";
    header.appendChild(h3);

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "×";
    closeBtn.style.background = "transparent";
    closeBtn.style.border = "none";
    closeBtn.style.color = "#aaa";
    closeBtn.style.fontSize = "20px";
    closeBtn.style.cursor = "pointer";
    closeBtn.addEventListener("click", () => {
      this.isOpen = false;
      if (this.onClose) this.onClose();
      backdrop.remove();
    });
    header.appendChild(closeBtn);

    dialog.appendChild(header);

    if (this.contentElement) {
      dialog.appendChild(this.contentElement);
    }

    backdrop.appendChild(dialog);
    container.appendChild(backdrop);
  }
}
