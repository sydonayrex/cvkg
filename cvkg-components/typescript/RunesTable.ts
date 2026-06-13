/**
 * Column definition for a RunesTable.
 */
export interface RunesTableColumn<D> {
  header: string;
  width: number;
  sortable?: boolean;
  cellBuilder: (item: D) => string;
}

/**
 * RunesTable component options/props.
 */
export interface RunesTableProps<D> {
  data: D[];
  columns: RunesTableColumn<D>[];
  rowHeight?: number;
  inlineEditable?: boolean;
  onEditCommit?: (rowIndex: number, colHeader: string, value: string) => void;
  onSelect?: (rowIndex: number) => void;
}

/**
 * A portable representation of the RunesTable data grid component.
 */
export class RunesTable<D> {
  private data: D[];
  private columns: RunesTableColumn<D>[];
  private rowHeight: number;
  private inlineEditable: boolean;
  private onEditCommit?: (rowIndex: number, colHeader: string, value: string) => void;
  private onSelect?: (rowIndex: number) => void;

  private selectedIndex: number | null = null;

  /**
   * Constructs a new RunesTable instance.
   */
  constructor(props: RunesTableProps<D>) {
    this.data = props.data;
    this.columns = props.columns;
    this.rowHeight = props.rowHeight ?? 32;
    this.inlineEditable = props.inlineEditable ?? false;
    this.onEditCommit = props.onEditCommit;
    this.onSelect = props.onSelect;
  }

  /**
   * Renders the table grid into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const table = document.createElement("table");
    table.style.width = "100%";
    table.style.borderCollapse = "collapse";
    table.style.fontFamily = "sans-serif";

    // Draw headers
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");
    this.columns.forEach(col => {
      const th = document.createElement("th");
      th.textContent = col.header;
      th.style.width = `${col.width}px`;
      th.style.padding = "8px";
      th.style.textAlign = "left";
      th.style.background = "#18181c";
      th.style.color = "#fff";
      th.style.border = "1px solid #333";
      headerRow.appendChild(th);
    });
    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Draw rows
    const tbody = document.createElement("tbody");
    this.data.forEach((item, rIdx) => {
      const row = document.createElement("tr");
      row.style.height = `${this.rowHeight}px`;
      
      const isSelected = this.selectedIndex === rIdx;
      row.style.background = isSelected ? "rgba(0, 100, 200, 0.4)" : (rIdx % 2 === 0 ? "#111" : "#1e1e24");
      row.style.cursor = "pointer";

      row.addEventListener("click", () => {
        this.selectedIndex = rIdx;
        if (this.onSelect) {
          this.onSelect(rIdx);
        }
        this.render(container);
      });

      this.columns.forEach((col, cIdx) => {
        const td = document.createElement("td");
        td.style.padding = "8px";
        td.style.border = "1px solid #333";
        td.style.color = "#fff";

        const isEditing = this.inlineEditable && isSelected && cIdx === 0;

        if (isEditing) {
          const input = document.createElement("input");
          input.type = "text";
          input.value = col.cellBuilder(item);
          input.style.width = "100%";
          input.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
              if (this.onEditCommit) {
                this.onEditCommit(rIdx, col.header, input.value);
              }
              this.render(container);
            }
          });
          td.appendChild(input);
        } else {
          td.innerHTML = col.cellBuilder(item);
        }
        row.appendChild(td);
      });
      tbody.appendChild(row);
    });
    table.appendChild(tbody);
    container.appendChild(table);
  }
}
