/**
 * Editable component for inline text editing.
 * 
 * Provides a toggleable label that mutates into a text input field on double-click or click.
 */
export interface EditableProps {
  /** The initial text value. */
  text?: string;
  /** Callback triggered when the text is committed/saved. */
  onCommit?: (val: string) => void;
}

/**
 * Portable representation of the Editable component.
 */
export class Editable {
  private text: string;
  private isEditing: boolean = false;
  private onCommit?: (val: string) => void;

  /**
   * Constructs a new Editable instance.
   */
  constructor(props: EditableProps = {}) {
    this.text = props.text ?? "";
    this.onCommit = props.onCommit;
  }

  /**
   * Toggles editing mode and redraws.
   */
  public setEditing(editing: boolean, container: HTMLElement): void {
    this.isEditing = editing;
    this.render(container);
  }

  /**
   * Renders the component into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    if (this.isEditing) {
      const input = document.createElement("input");
      input.type = "text";
      input.value = this.text;
      input.style.width = "100%";
      input.style.padding = "8px";
      input.style.borderRadius = "4px";
      
      const commit = () => {
        this.text = input.value;
        if (this.onCommit) {
          this.onCommit(this.text);
        }
        this.setEditing(false, container);
      };

      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          commit();
        }
      });
      input.addEventListener("blur", commit);

      container.appendChild(input);
      input.focus();
    } else {
      const span = document.createElement("span");
      span.textContent = this.text || "Click to edit...";
      span.style.cursor = "pointer";
      span.style.padding = "4px 8px";
      span.style.borderRadius = "4px";

      span.addEventListener("click", () => {
        this.setEditing(true, container);
      });

      container.appendChild(span);
    }
  }
}
