/**
 * Button component for user action triggers.
 * 
 * Handles styling variants and click handlers.
 */
export interface ButtonProps {
  /** Text shown inside the button. */
  label: string;
  /** Callback triggered on button click. */
  onClick?: () => void;
  /** Style layout mode. */
  variant?: "primary" | "secondary" | "danger" | "ghost";
}

/**
 * Portable representation of the Button component.
 */
export class Button {
  private label: string;
  private onClick?: () => void;
  private variant: string;

  /**
   * Constructs a new Button instance.
   */
  constructor(props: ButtonProps) {
    this.label = props.label;
    this.onClick = props.onClick;
    this.variant = props.variant ?? "primary";
  }

  /**
   * Renders the button element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const btn = document.createElement("button");
    btn.textContent = this.label;
    btn.style.padding = "8px 16px";
    btn.style.borderRadius = "4px";
    btn.style.cursor = "pointer";
    btn.style.border = "none";
    btn.style.fontWeight = "600";

    if (this.variant === "primary") {
      btn.style.background = "#0080ff";
      btn.style.color = "#fff";
    } else if (this.variant === "secondary") {
      btn.style.background = "#333";
      btn.style.color = "#ccc";
      btn.style.border = "1px solid #555";
    } else if (this.variant === "danger") {
      btn.style.background = "#ff4d4d";
      btn.style.color = "#fff";
    } else if (this.variant === "ghost") {
      btn.style.background = "transparent";
      btn.style.color = "#fff";
    }

    if (this.onClick) {
      btn.addEventListener("click", this.onClick);
    }

    container.appendChild(btn);
  }
}
