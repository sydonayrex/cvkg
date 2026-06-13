/**
 * Checkbox component for binary options states.
 * 
 * Provides label support and toggle state.
 */
export interface CheckboxProps {
  /** Text shown alongside the checkbox box. */
  label?: string;
  /** Whether the checkbox box is currently checked. */
  checked?: boolean;
  /** Callback triggered when state toggles. */
  onChange?: (checked: boolean) => void;
}

/**
 * Portable representation of the Checkbox component.
 */
export class Checkbox {
  private label: string;
  private checked: boolean;
  private onChange?: (checked: boolean) => void;

  /**
   * Constructs a new Checkbox instance.
   */
  constructor(props: CheckboxProps = {}) {
    this.label = props.label ?? "";
    this.checked = props.checked ?? false;
    this.onChange = props.onChange;
  }

  /**
   * Renders the checkbox element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("label");
    wrapper.style.display = "inline-flex";
    wrapper.style.alignItems = "center";
    wrapper.style.gap = "8px";
    wrapper.style.cursor = "pointer";

    const input = document.createElement("input");
    input.type = "checkbox";
    input.checked = this.checked;

    input.addEventListener("change", (e) => {
      this.checked = (e.target as HTMLInputElement).checked;
      if (this.onChange) {
        this.onChange(this.checked);
      }
    });

    const text = document.createElement("span");
    text.textContent = this.label;
    text.style.color = "#fff";

    wrapper.appendChild(input);
    if (this.label) {
      wrapper.appendChild(text);
    }
    container.appendChild(wrapper);
  }
}
