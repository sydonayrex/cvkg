/**
 * Sonner component rendering toast notification alerts.
 * 
 * Provides stackable toast notifications.
 */
export interface SonnerToast {
  id: string;
  title: string;
  description?: string;
  duration?: number;
}

/**
 * Portable representation of the Sonner stackable toast notifications manager.
 */
export class Sonner {
  private containerEl?: HTMLElement;
  private toasts: SonnerToast[] = [];

  /**
   * Constructs a new Sonner toast manager instance.
   */
  constructor() {
    this.createContainer();
  }

  /**
   * Setup container element on DOM document root.
   */
  private createContainer(): void {
    let container = document.getElementById("sonner-toast-container");
    if (!container) {
      container = document.createElement("div");
      container.id = "sonner-toast-container";
      container.style.position = "fixed";
      container.style.bottom = "16px";
      container.style.right = "16px";
      container.style.display = "flex";
      container.style.flexDirection = "column";
      container.style.gap = "8px";
      container.style.zIndex = "2000";
      document.body.appendChild(container);
    }
    this.containerEl = container;
  }

  /**
   * Pushes a new alert notification toast onto the stack.
   */
  public toast(title: string, description?: string, duration = 3000): void {
    const id = Math.random().toString(36).substring(2, 9);
    const toastItem: SonnerToast = { id, title, description, duration };
    this.toasts.push(toastItem);

    const toastNode = document.createElement("div");
    toastNode.style.background = "#18181c";
    toastNode.style.border = "1px solid #333";
    toastNode.style.borderRadius = "6px";
    toastNode.style.padding = "12px 16px";
    toastNode.style.width = "260px";
    toastNode.style.boxShadow = "0 4px 6px rgba(0,0,0,0.3)";
    toastNode.style.color = "#fff";

    const h = document.createElement("div");
    h.style.fontWeight = "bold";
    h.style.fontSize = "13px";
    h.textContent = title;
    toastNode.appendChild(h);

    if (description) {
      const d = document.createElement("div");
      d.style.fontSize = "11px";
      d.style.color = "#aaa";
      d.style.marginTop = "4px";
      d.textContent = description;
      toastNode.appendChild(d);
    }

    if (this.containerEl) {
      this.containerEl.appendChild(toastNode);
    }

    setTimeout(() => {
      toastNode.remove();
      this.toasts = this.toasts.filter(t => t.id !== id);
    }, duration);
  }
}
