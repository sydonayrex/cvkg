/**
 * Avatar component for user profiles.
 * 
 * Renders fallback initials or avatar images.
 */
export interface AvatarProps {
  /** Optional source URL of the avatar image. */
  src?: string;
  /** Fallback initials to display (e.g. "JD"). */
  initials?: string;
  /** Sizing of the avatar sphere. */
  size?: "sm" | "md" | "lg";
}

/**
 * Portable representation of the Avatar component.
 */
export class Avatar {
  private src?: string;
  private initials: string;
  private size: string;

  /**
   * Constructs a new Avatar instance.
   */
  constructor(props: AvatarProps = {}) {
    this.src = props.src;
    this.initials = props.initials ?? "U";
    this.size = props.size ?? "md";
  }

  /**
   * Renders the avatar element inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const av = document.createElement("div");
    av.style.borderRadius = "50%";
    av.style.display = "flex";
    av.style.alignItems = "center";
    av.style.justifyContent = "center";
    av.style.background = "#44475a";
    av.style.color = "#fff";
    av.style.fontWeight = "bold";
    av.style.overflow = "hidden";

    let dimension = 40;
    if (this.size === "sm") {
      dimension = 32;
    } else if (this.size === "lg") {
      dimension = 56;
    }

    av.style.width = `${dimension}px`;
    av.style.height = `${dimension}px`;
    av.style.fontSize = `${dimension * 0.4}px`;

    if (this.src) {
      const img = document.createElement("img");
      img.src = this.src;
      img.style.width = "100%";
      img.style.height = "100%";
      img.style.objectFit = "cover";
      av.appendChild(img);
    } else {
      av.textContent = this.initials;
    }

    container.appendChild(av);
  }
}
