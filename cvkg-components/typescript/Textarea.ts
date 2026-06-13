/**
 * Textarea component for multi-line inputs.
 * 
 * Provides interactive text binding events.
 */
export interface TextareaProps {
  /** Placeholder text. */
  placeholder?: string;
  /** Current text value. */
  value?: string;
  /** Callback triggered on text change events. */
  onChange?: (val: string) => void;
}

/**
 * Portable representation of the Textarea component.
 */
export class Textarea {
  private placeholder: string;
  private value: string;
  private onChange?: (val: string) => void;

  /**
   * Constructs a new Textarea instance.
   */
  constructor(props: TextareaProps = {}) {
    this.placeholder = props.placeholder ?? "";
    this.value = props.value ?? "";
    this.onChange = props.onChange;
  }

  /**
   * Renders the textarea element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const area = document.createElement("textarea");
    area.placeholder = this.placeholder;
    area.value = this.value;
    area.style.width = "100%";
    area.style.minHeight = "80px";
    area.style.padding = "8px";
    area.style.borderRadius = "4px";
    area.style.border = "1px solid #333";
    area.style.background = "#1e1e24";
    area.style.color = "#fff";

    area.addEventListener("input", (e) => {
      this.value = (e.target as HTMLTextAreaElement).value;
      if (this.onChange) {
        this.onChange(this.value);
      }
    });

    container.appendChild(area);
  }
}
