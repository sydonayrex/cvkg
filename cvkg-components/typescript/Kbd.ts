/**
 * Kbd component representing keyboard shortcut layouts.
 * 
 * Styled for rendering standard hotkey indicator labels.
 */
export interface KbdProps {
  /** The key string (e.g. "Ctrl", "Enter"). */
  keyLabel: string;
}

/**
 * Portable representation of the Kbd component.
 */
export class Kbd {
  private keyLabel: string;

  /**
   * Constructs a new Kbd instance.
   */
  constructor(props: KbdProps) {
    this.keyLabel = props.keyLabel;
  }

  /**
   * Renders the hotkey element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const kbd = document.createElement("kbd");
    kbd.textContent = this.keyLabel;
    kbd.style.padding = "2px 6px";
    kbd.style.fontSize = "11px";
    kbd.style.fontFamily = "monospace";
    kbd.style.background = "#282a36";
    kbd.style.border = "1px solid #44475a";
    kbd.style.borderRadius = "3px";
    kbd.style.color = "#f8f8f2";
    kbd.style.boxShadow = "0 1px 0 rgba(0,0,0,0.2)";

    container.appendChild(kbd);
  }
}
