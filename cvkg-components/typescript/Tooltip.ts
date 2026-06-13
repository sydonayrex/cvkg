/**
 * Tooltip component displaying hover contextual labels.
 * 
 * Anchors text blocks on hover.
 */
export interface TooltipProps {
  /** The text inside the popover. */
  text: string;
}

/**
 * Portable representation of the Tooltip component.
 */
export class Tooltip {
  private text: string;

  /**
   * Constructs a new Tooltip instance.
   */
  constructor(props: TooltipProps) {
    this.text = props.text;
  }

  /**
   * Attaches hover trigger tooltip overlays to an element.
   */
  public attach(target: HTMLElement): void {
    target.style.position = "relative";

    const tip = document.createElement("div");
    tip.textContent = this.text;
    tip.style.position = "absolute";
    tip.style.bottom = "100%";
    tip.style.left = "50%";
    tip.style.transform = "translateX(-50%) translateY(-4px)";
    tip.style.padding = "4px 8px";
    tip.style.background = "rgba(0,0,0,0.85)";
    tip.style.color = "#fff";
    tip.style.fontSize = "11px";
    tip.style.borderRadius = "4px";
    tip.style.whiteSpace = "nowrap";
    tip.style.display = "none";
    tip.style.pointerEvents = "none";
    tip.style.zIndex = "1000";

    target.appendChild(tip);

    target.addEventListener("mouseenter", () => {
      tip.style.display = "block";
    });
    target.addEventListener("mouseleave", () => {
      tip.style.display = "none";
    });
  }
}
