/**
 * Toggle Switch component for binary states.
 * 
 * Renders a visual slider pill switch.
 */
export interface ToggleProps {
  /** Whether the switch is currently on. */
  isOn?: boolean;
  /** Callback triggered when state toggles. */
  onChange?: (isOn: boolean) => void;
}

/**
 * Portable representation of the Toggle Switch.
 */
export class Toggle {
  private isOn: boolean;
  private onChange?: (isOn: boolean) => void;

  /**
   * Constructs a new Toggle Switch instance.
   */
  constructor(props: ToggleProps = {}) {
    this.isOn = props.isOn ?? false;
    this.onChange = props.onChange;
  }

  /**
   * Renders the toggle switch element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const switchEl = document.createElement("div");
    switchEl.style.width = "40px";
    switchEl.style.height = "20px";
    switchEl.style.borderRadius = "10px";
    switchEl.style.background = this.isOn ? "#0080ff" : "#333";
    switchEl.style.position = "relative";
    switchEl.style.cursor = "pointer";
    switchEl.style.transition = "background-color 0.2s";

    const knob = document.createElement("div");
    knob.style.width = "16px";
    knob.style.height = "16px";
    knob.style.borderRadius = "50%";
    knob.style.background = "#fff";
    knob.style.position = "absolute";
    knob.style.top = "2px";
    knob.style.left = this.isOn ? "22px" : "2px";
    knob.style.transition = "left 0.2s";

    switchEl.appendChild(knob);

    switchEl.addEventListener("click", () => {
      this.isOn = !this.isOn;
      switchEl.style.background = this.isOn ? "#0080ff" : "#333";
      knob.style.left = this.isOn ? "22px" : "2px";
      if (this.onChange) {
        this.onChange(this.isOn);
      }
    });

    container.appendChild(switchEl);
  }
}
