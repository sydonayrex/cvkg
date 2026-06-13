/**
 * MentionInput component featuring autocomplete suggestions for '@' and '#' tags.
 * 
 * Monitored input text to render inline tag popovers.
 */
export interface MentionInputProps {
  /** The initial text value. */
  text?: string;
  /** Users list for autocomplete suggestions triggered by '@'. */
  users?: string[];
  /** Topics list for autocomplete suggestions triggered by '#'. */
  topics?: string[];
  /** Callback triggered when the text value changes. */
  onChange?: (value: string) => void;
}

/**
 * Portable representation of the MentionInput component.
 */
export class MentionInput {
  private text: string;
  private users: string[];
  private topics: string[];
  private onChange?: (value: string) => void;

  /**
   * Constructs a new MentionInput instance.
   */
  constructor(props: MentionInputProps = {}) {
    this.text = props.text ?? "";
    this.users = props.users ?? ["alice", "bob", "charlie"];
    this.topics = props.topics ?? ["rust", "ui", "gpu"];
    this.onChange = props.onChange;
  }

  /**
   * Sets the input text value and notifies handlers.
   */
  public setText(val: string): void {
    this.text = val;
    if (this.onChange) {
      this.onChange(val);
    }
  }

  /**
   * Renders the component into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.position = "relative";
    wrapper.style.width = "100%";

    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "Type here... Use @name or #topic";
    input.value = this.text;
    input.style.width = "100%";
    input.style.padding = "8px";
    input.style.borderRadius = "4px";

    const dropdown = document.createElement("div");
    dropdown.style.position = "absolute";
    dropdown.style.top = "100%";
    dropdown.style.left = "8px";
    dropdown.style.width = "180px";
    dropdown.style.background = "#1e1e24";
    dropdown.style.border = "1px solid #333";
    dropdown.style.borderRadius = "4px";
    dropdown.style.display = "none";
    dropdown.style.zIndex = "100";

    const showSuggestions = (items: string[], symbol: string) => {
      dropdown.innerHTML = "";
      dropdown.style.display = "block";
      items.forEach(item => {
        const row = document.createElement("div");
        row.style.padding = "8px";
        row.style.cursor = "pointer";
        row.style.color = "#fff";
        row.textContent = item;
        
        row.addEventListener("click", () => {
          const base = this.text.substring(0, this.text.length - 1);
          this.setText(`${base}${item} `);
          input.value = this.text;
          dropdown.style.display = "none";
        });
        dropdown.appendChild(row);
      });
    };

    input.addEventListener("input", (e) => {
      const val = (e.target as HTMLInputElement).value;
      this.setText(val);

      if (val.endsWith("@")) {
        showSuggestions(this.users, "@");
      } else if (val.endsWith("#")) {
        showSuggestions(this.topics, "#");
      } else {
        dropdown.style.display = "none";
      }
    });

    wrapper.appendChild(input);
    wrapper.appendChild(dropdown);
    container.appendChild(wrapper);
  }
}
