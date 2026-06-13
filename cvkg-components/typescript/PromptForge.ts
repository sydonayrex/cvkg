/**
 * Segment templates definitions inside the prompt builder.
 */
export interface PromptSegment {
  id: string;
  name: string;
  template: string;
}

export interface PromptForgeProps {
  /** The list of selectable system segment modules. */
  segments: PromptSegment[];
  /** Callback triggered when a prompt is compiled. */
  onForge?: (compiledPrompt: string) => void;
}

/**
 * Portable representation of the PromptForge template builder.
 */
export class PromptForge {
  private segments: PromptSegment[];
  private onForge?: (compiledPrompt: string) => void;

  /**
   * Constructs a new PromptForge instance.
   */
  constructor(props: PromptForgeProps) {
    this.segments = props.segments;
    this.onForge = props.onForge;
  }

  /**
   * Renders the template forge builder into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.display = "flex";
    wrapper.style.flexDirection = "column";
    wrapper.style.gap = "12px";
    wrapper.style.padding = "16px";
    wrapper.style.background = "#18181c";
    wrapper.style.border = "1px solid #333";
    wrapper.style.borderRadius = "8px";

    const title = document.createElement("h3");
    title.textContent = "Prompt Forge Studio";
    title.style.color = "#fff";
    title.style.margin = "0";
    wrapper.appendChild(title);

    const selectionArea = document.createElement("div");
    selectionArea.style.display = "flex";
    selectionArea.style.flexDirection = "column";
    selectionArea.style.gap = "8px";

    const textareas: HTMLTextAreaElement[] = [];

    this.segments.forEach(seg => {
      const row = document.createElement("div");
      row.style.display = "flex";
      row.style.flexDirection = "column";
      row.style.gap = "4px";

      const label = document.createElement("label");
      label.textContent = seg.name;
      label.style.fontSize = "12px";
      label.style.color = "#aaa";

      const area = document.createElement("textarea");
      area.value = seg.template;
      area.dataset.id = seg.id;
      area.style.minHeight = "44px";
      area.style.padding = "6px";
      area.style.background = "#1e1e24";
      area.style.border = "1px solid #444";
      area.style.borderRadius = "4px";
      area.style.color = "#fff";
      
      row.appendChild(label);
      row.appendChild(area);
      selectionArea.appendChild(row);
      textareas.push(area);
    });
    wrapper.appendChild(selectionArea);

    const forgeBtn = document.createElement("button");
    forgeBtn.textContent = "Compile Prompt";
    forgeBtn.style.padding = "8px 16px";
    forgeBtn.style.background = "#2ea44f";
    forgeBtn.style.color = "#fff";
    forgeBtn.style.border = "none";
    forgeBtn.style.borderRadius = "4px";
    forgeBtn.style.cursor = "pointer";

    forgeBtn.addEventListener("click", () => {
      const segmentsText = textareas.map(ta => ta.value).join("\n\n");
      if (this.onForge) {
        this.onForge(segmentsText);
      }
    });

    wrapper.appendChild(forgeBtn);
    container.appendChild(wrapper);
  }
}
