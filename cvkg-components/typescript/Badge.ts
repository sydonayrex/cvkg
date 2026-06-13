/**
 * Badge component for small status pill counts.
 * 
 * Provides notification pill labels.
 */
export interface BadgeProps {
  /** Text label or count inside the badge. */
  label: string;
  /** Background theme color helper. */
  variant?: "info" | "success" | "warning" | "danger";
}

/**
 * Portable representation of the Badge component.
 */
export class Badge {
  private label: string;
  private variant: string;

  /**
   * Constructs a new Badge instance.
   */
  constructor(props: BadgeProps) {
    this.label = props.label;
    this.variant = props.variant ?? "info";
  }

  /**
   * Renders the badge element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const badge = document.createElement("span");
    badge.textContent = this.label;
    badge.style.padding = "2px 8px";
    badge.style.borderRadius = "12px";
    badge.style.fontSize = "11px";
    badge.style.fontWeight = "bold";
    badge.style.color = "#fff";
    badge.style.display = "inline-block";

    if (this.variant === "info") {
      badge.style.background = "#0080ff";
    } else if (this.variant === "success") {
      badge.style.background = "#2ea44f";
    } else if (this.variant === "warning") {
      badge.style.background = "#ff9900";
    } else if (this.variant === "danger") {
      badge.style.background = "#ff3333";
    }

    container.appendChild(badge);
  }
}
