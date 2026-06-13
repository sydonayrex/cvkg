/**
 * TreeView component representing hierarchical expandable lists.
 * 
 * Provides collapsible node trees.
 */
export interface TreeNode {
  id: string;
  label: string;
  children?: TreeNode[];
}

export interface TreeViewProps {
  /** Root hierarchical list of nodes. */
  nodes: TreeNode[];
  /** Callback triggered when a terminal node is clicked. */
  onNodeClick?: (nodeId: string) => void;
}

/**
 * Portable representation of the TreeView component.
 */
export class TreeView {
  private nodes: TreeNode[];
  private onNodeClick?: (nodeId: string) => void;

  /**
   * Constructs a new TreeView instance.
   */
  constructor(props: TreeViewProps) {
    this.nodes = props.nodes;
    this.onNodeClick = props.onNodeClick;
  }

  /**
   * Recursively builds tree nodes elements.
   */
  private buildNode(node: TreeNode): HTMLElement {
    const item = document.createElement("div");
    item.style.paddingLeft = "16px";
    item.style.fontFamily = "monospace";
    item.style.fontSize = "13px";
    item.style.color = "#fff";

    const labelRow = document.createElement("div");
    labelRow.style.cursor = "pointer";
    labelRow.style.display = "flex";
    labelRow.style.alignItems = "center";
    labelRow.style.gap = "6px";
    labelRow.style.padding = "4px 0";

    const hasChildren = node.children && node.children.length > 0;
    
    if (hasChildren) {
      const toggle = document.createElement("span");
      toggle.textContent = "▶";
      toggle.style.fontSize = "10px";
      toggle.style.color = "#aaa";
      labelRow.appendChild(toggle);

      const childrenContainer = document.createElement("div");
      childrenContainer.style.display = "none";
      
      node.children?.forEach(child => {
        childrenContainer.appendChild(this.buildNode(child));
      });

      labelRow.addEventListener("click", (e) => {
        e.stopPropagation();
        const isCollapsed = childrenContainer.style.display === "none";
        childrenContainer.style.display = isCollapsed ? "block" : "none";
        toggle.textContent = isCollapsed ? "▼" : "▶";
      });

      item.appendChild(labelRow);
      item.appendChild(childrenContainer);
    } else {
      const bullet = document.createElement("span");
      bullet.textContent = "•";
      bullet.style.color = "#444";
      labelRow.appendChild(bullet);

      const text = document.createElement("span");
      text.textContent = node.label;
      labelRow.appendChild(text);

      labelRow.addEventListener("click", (e) => {
        e.stopPropagation();
        if (this.onNodeClick) {
          this.onNodeClick(node.id);
        }
      });
      item.appendChild(labelRow);
    }

    return item;
  }

  /**
   * Renders the tree elements inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const wrapper = document.createElement("div");
    wrapper.style.display = "flex";
    wrapper.style.flexDirection = "column";

    this.nodes.forEach(node => {
      wrapper.appendChild(this.buildNode(node));
    });

    container.appendChild(wrapper);
  }
}
