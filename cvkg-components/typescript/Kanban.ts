/**
 * Kanban board component for workflow organization status columns.
 * 
 * Provides interactive column status drag-and-drop.
 */
export interface KanbanCard {
  id: string;
  title: string;
  description?: string;
}

export interface KanbanColumn {
  id: string;
  title: string;
  cards: KanbanCard[];
}

export interface KanbanProps {
  /** Column statuses definitions. */
  columns: KanbanColumn[];
  /** Callback triggered when card drops on another column. */
  onCardMove?: (cardId: string, fromColId: string, toColId: string) => void;
}

/**
 * Portable representation of the Kanban board component.
 */
export class Kanban {
  private columns: KanbanColumn[];
  private onCardMove?: (cardId: string, fromColId: string, toColId: string) => void;

  /**
   * Constructs a new Kanban instance.
   */
  constructor(props: KanbanProps) {
    this.columns = props.columns;
    this.onCardMove = props.onCardMove;
  }

  /**
   * Renders the board grid columns into a DOM element container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const board = document.createElement("div");
    board.style.display = "flex";
    board.style.gap = "16px";
    board.style.width = "100%";
    board.style.overflowX = "auto";
    board.style.padding = "16px";

    this.columns.forEach(col => {
      const colEl = document.createElement("div");
      colEl.style.width = "250px";
      colEl.style.background = "#18181c";
      colEl.style.border = "1px solid #333";
      colEl.style.borderRadius = "6px";
      colEl.style.padding = "12px";
      colEl.style.display = "flex";
      colEl.style.flexDirection = "column";
      colEl.style.gap = "8px";

      const title = document.createElement("h4");
      title.textContent = col.title;
      title.style.color = "#fff";
      title.style.margin = "0 0 8px 0";
      colEl.appendChild(title);

      col.cards.forEach(card => {
        const cardEl = document.createElement("div");
        cardEl.style.background = "#1e1e24";
        cardEl.style.border = "1px solid #444";
        cardEl.style.borderRadius = "4px";
        cardEl.style.padding = "8px";
        cardEl.style.cursor = "grab";
        cardEl.draggable = true;

        const h = document.createElement("div");
        h.style.fontWeight = "bold";
        h.style.fontSize = "13px";
        h.style.color = "#fff";
        h.textContent = card.title;
        cardEl.appendChild(h);

        if (card.description) {
          const d = document.createElement("div");
          d.style.fontSize = "11px";
          d.style.color = "#aaa";
          d.style.marginTop = "4px";
          d.textContent = card.description;
          cardEl.appendChild(d);
        }

        cardEl.addEventListener("dragstart", (e) => {
          e.dataTransfer?.setData("text/plain", JSON.stringify({ cardId: card.id, fromColId: col.id }));
        });

        colEl.appendChild(cardEl);
      });

      colEl.addEventListener("dragover", (e) => {
        e.preventDefault();
      });

      colEl.addEventListener("drop", (e) => {
        e.preventDefault();
        const dataStr = e.dataTransfer?.getData("text/plain");
        if (dataStr) {
          const { cardId, fromColId } = JSON.parse(dataStr);
          if (fromColId !== col.id && this.onCardMove) {
            this.onCardMove(cardId, fromColId, col.id);
          }
        }
      });

      board.appendChild(colEl);
    });

    container.appendChild(board);
  }
}
