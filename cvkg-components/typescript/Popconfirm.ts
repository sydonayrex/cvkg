/**
 * Popconfirm component for lightweight inline confirmation overlays.
 * 
 * Provides quick Yes/No actions relative to an anchor target.
 */
export interface PopconfirmProps {
  /** The message/question shown in the confirmation box. */
  message: string;
  /** Callback triggered if the user confirms the action. */
  onConfirm?: () => void;
}

/**
 * Portable representation of the Popconfirm component.
 */
export class Popconfirm {
  private message: string;
  private onConfirm?: () => void;
  private isOpen: boolean = false;

  /**
   * Constructs a new Popconfirm instance.
   */
  constructor(props: PopconfirmProps) {
    this.message = props.message;
    this.onConfirm = props.onConfirm;
  }

  /**
   * Renders the component and attaches it to the trigger element.
   */
  public attach(trigger: HTMLElement): void {
    trigger.style.position = "relative";
    trigger.style.cursor = "pointer";

    const popover = document.createElement("div");
    popover.style.position = "absolute";
    popover.style.top = "100%";
    popover.style.left = "0px";
    popover.style.width = "180px";
    popover.style.padding = "8px";
    popover.style.background = "#1e1e24";
    popover.style.border = "1px solid #333";
    popover.style.borderRadius = "4px";
    popover.style.display = "none";
    popover.style.zIndex = "200";

    const msg = document.createElement("div");
    msg.style.fontSize = "12px";
    msg.style.color = "#fff";
    msg.style.marginBottom = "8px";
    msg.textContent = this.message;
    popover.appendChild(msg);

    const actions = document.createElement("div");
    actions.style.display = "flex";
    actions.style.gap = "8px";

    const yesBtn = document.createElement("button");
    yesBtn.textContent = "Yes";
    yesBtn.style.flex = "1";
    yesBtn.style.padding = "4px";
    yesBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      if (this.onConfirm) {
        this.onConfirm();
      }
      this.isOpen = false;
      popover.style.display = "none";
    });

    const noBtn = document.createElement("button");
    noBtn.textContent = "No";
    noBtn.style.flex = "1";
    noBtn.style.padding = "4px";
    noBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      this.isOpen = false;
      popover.style.display = "none";
    });

    actions.appendChild(yesBtn);
    actions.appendChild(noBtn);
    popover.appendChild(actions);
    trigger.appendChild(popover);

    trigger.addEventListener("click", () => {
      this.isOpen = !this.isOpen;
      popover.style.display = this.isOpen ? "block" : "none";
    });
  }
}
