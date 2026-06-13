/**
 * Input component for text input fields.
 * 
 * Provides interactive text binding events.
 */
export interface InputProps {
  /** Text shown when input is empty. */
  placeholder?: string;
  /** Current text value. */
  value?: string;
  /** Callback triggered on input change events. */
  onChange?: (val: string) => void;
}

/**
 * Portable representation of the Input component.
 */
export class Input {
  private placeholder: string;
  private value: string;
  private onChange?: (val: string) => void;

  /**
   * Constructs a new Input instance.
   */
  constructor(props: InputProps = {}) {
    this.placeholder = props.placeholder ?? "";
    this.value = props.value ?? "";
    this.onChange = props.onChange;
  }

  /**
   * Renders the input element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = this.placeholder;
    input.value = this.value;
    input.style.width = "100%";
    input.style.padding = "8px";
    input.style.borderRadius = "4px";
    input.style.border = "1px solid #333";
    input.style.background = "#1e1e24";
    input.style.color = "#fff";

    input.addEventListener("input", (e) => {
      this.value = (e.target as HTMLInputElement).value;
      if (this.onChange) {
        this.onChange(this.value);
      }
    });

    container.appendChild(input);
  }
}
