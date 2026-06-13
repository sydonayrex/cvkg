/**
 * Slider component for range values selection.
 * 
 * Provides interactive value selection.
 */
export interface SliderProps {
  /** The current numeric value. */
  value?: number;
  /** Minimum slider value boundary. */
  min?: number;
  /** Maximum slider value boundary. */
  max?: number;
  /** Steps granularity. */
  step?: number;
  /** Callback triggered when value shifts. */
  onChange?: (val: number) => void;
}

/**
 * Portable representation of the Slider range input.
 */
export class Slider {
  private value: number;
  private min: number;
  private max: number;
  private step: number;
  private onChange?: (val: number) => void;

  /**
   * Constructs a new Slider instance.
   */
  constructor(props: SliderProps = {}) {
    this.value = props.value ?? 50;
    this.min = props.min ?? 0;
    this.max = props.max ?? 100;
    this.step = props.step ?? 1;
    this.onChange = props.onChange;
  }

  /**
   * Renders the slider element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const slider = document.createElement("input");
    slider.type = "range";
    slider.min = `${this.min}`;
    slider.max = `${this.max}`;
    slider.step = `${this.step}`;
    slider.value = `${this.value}`;
    slider.style.width = "100%";

    slider.addEventListener("input", (e) => {
      this.value = parseFloat((e.target as HTMLInputElement).value);
      if (this.onChange) {
        this.onChange(this.value);
      }
    });

    container.appendChild(slider);
  }
}
