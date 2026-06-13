/**
 * PhoneInput component combining a country-code selector and number input.
 * 
 * Provides a standardized layout for international telephone number input.
 */
export interface PhoneInputProps {
  /** The current country code (e.g. "+1", "+44"). */
  countryCode?: string;
  /** The current phone number string. */
  phoneNumber?: string;
  /** Callback triggered when either value changes. */
  onChange?: (countryCode: string, phoneNumber: string) => void;
}

/**
 * Portable representation of the PhoneInput component.
 */
export class PhoneInput {
  private countryCode: string;
  private phoneNumber: string;
  private onChange?: (countryCode: string, phoneNumber: string) => void;

  /**
   * Constructs a new PhoneInput instance.
   * 
   * # Contract
   * - Default country code is "+1" if not specified.
   */
  constructor(props: PhoneInputProps = {}) {
    this.countryCode = props.countryCode ?? "+1";
    this.phoneNumber = props.phoneNumber ?? "";
    this.onChange = props.onChange;
  }

  /**
   * Triggers the onChange handler with updated values.
   */
  private triggerChange(): void {
    if (this.onChange) {
      this.onChange(this.countryCode, this.phoneNumber);
    }
  }

  /**
   * Sets the country code value and notifies handlers.
   */
  public setCountryCode(code: string): void {
    this.countryCode = code;
    this.triggerChange();
  }

  /**
   * Sets the phone number value and notifies handlers.
   */
  public setPhoneNumber(num: string): void {
    this.phoneNumber = num;
    this.triggerChange();
  }

  /**
   * Renders the component into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    const wrapper = document.createElement("div");
    wrapper.style.display = "flex";
    wrapper.style.gap = "8px";
    wrapper.style.width = "100%";

    const select = document.createElement("select");
    select.style.padding = "8px";
    select.style.borderRadius = "4px";
    
    const options = ["+1", "+44", "+49", "+81", "+86"];
    options.forEach(opt => {
      const el = document.createElement("option");
      el.value = opt;
      el.textContent = opt;
      el.selected = opt === this.countryCode;
      select.appendChild(el);
    });

    select.addEventListener("change", (e) => {
      this.setCountryCode((e.target as HTMLSelectElement).value);
    });

    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "Phone Number";
    input.value = this.phoneNumber;
    input.style.flex = "1";
    input.style.padding = "8px";
    input.style.borderRadius = "4px";

    input.addEventListener("input", (e) => {
      this.setPhoneNumber((e.target as HTMLInputElement).value);
    });

    wrapper.appendChild(select);
    wrapper.appendChild(input);
    container.appendChild(wrapper);
  }
}
