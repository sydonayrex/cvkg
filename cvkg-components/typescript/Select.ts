/**
 * Select component for option list selection dropdowns.
 * 
 * Provides interactive option selection.
 */
export interface SelectOption {
  label: string;
  value: string;
}

export interface SelectProps {
  /** The selected initial value. */
  value?: string;
  /** Complete list of selectable options. */
  options: SelectOption[];
  /** Callback triggered when selection changes. */
  onChange?: (val: string) => void;
}

/**
 * Portable representation of the Select dropdown.
 */
export class Select {
  private value: string;
  private options: SelectOption[];
  private onChange?: (val: string) => void;

  /**
   * Constructs a new Select instance.
   */
  constructor(props: SelectProps) {
    this.value = props.value ?? "";
    this.options = props.options;
    this.onChange = props.onChange;
  }

  /**
   * Renders the select dropdown element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const select = document.createElement("select");
    select.style.width = "100%";
    select.style.padding = "8px";
    select.style.borderRadius = "4px";
    select.style.border = "1px solid #333";
    select.style.background = "#1e1e24";
    select.style.color = "#fff";

    this.options.forEach(opt => {
      const el = document.createElement("option");
      el.value = opt.value;
      el.textContent = opt.label;
      el.selected = opt.value === this.value;
      select.appendChild(el);
    });

    select.addEventListener("change", (e) => {
      this.value = (e.target as HTMLSelectElement).value;
      if (this.onChange) {
        this.onChange(this.value);
      }
    });

    container.appendChild(select);
  }
}
