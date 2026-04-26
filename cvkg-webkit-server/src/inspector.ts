/**
 * CVKG Agentic Development Guidelines (v1.2) - Guideline 6 Compliance
 * All public Inspector protocol functions have TypeScript JSDoc comments.
 */

export interface LayoutRect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface AriaProps {
    label?: string;
    disabled: boolean;
    hidden: boolean;
}

export interface VNode {
    id: number;
    key?: string;
    component_type: string;
    props: Record<string, any>;
    state?: Record<string, any>;
    layout: LayoutRect;
    children: number[];
    aria_role: string;
    aria_props: AriaProps;
}

/**
 * High-performance Inspector Overlay for real-time UI debugging.
 */
export class InspectorOverlay {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private nodes: Map<number, VNode> = new Map();
    private hoveredId: number | null = null;

    constructor() {
        this.canvas = document.createElement("canvas");
        this.canvas.style.position = "fixed";
        this.canvas.style.top = "0";
        this.canvas.style.left = "0";
        this.canvas.style.pointerEvents = "none";
        this.canvas.style.zIndex = "9999";
        document.body.appendChild(this.canvas);
        
        const ctx = this.canvas.getContext("2d");
        if (!ctx) throw new Error("Failed to get 2D context");
        this.ctx = ctx;

        this.resize();
        window.addEventListener("resize", () => this.resize());
    }

    private resize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }

    /**
     * Updates the local VNode cache with a new snapshot.
     */
    public updateSnapshot(nodes: VNode[]) {
        this.nodes.clear();
        nodes.forEach(n => this.nodes.set(n.id, n));
        this.render();
    }

    /**
     * Highlights a specific node on the overlay.
     */
    public highlight(id: number | null) {
        this.hoveredId = id;
        this.render();
    }

    private render() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        
        if (this.hoveredId !== null) {
            const node = this.nodes.get(this.hoveredId);
            if (node) {
                this.drawNodeHighlight(node, "rgba(0, 120, 255, 0.3)", "rgba(0, 120, 255, 0.8)");
            }
        }
    }

    private drawNodeHighlight(node: VNode, fill: string, stroke: string) {
        const { x, y, width, height } = node.layout;
        this.ctx.fillStyle = fill;
        this.ctx.strokeStyle = stroke;
        this.ctx.lineWidth = 2;
        this.ctx.fillRect(x, y, width, height);
        this.ctx.strokeRect(x, y, width, height);
        
        // Label
        this.ctx.fillStyle = "white";
        this.ctx.font = "12px monospace";
        const label = `${node.component_type} [${node.id}]`;
        this.ctx.fillText(label, x, y - 5);
    }
}

/**
 * State-Preserving HMR Handler.
 * Injects new WASM while keeping Niflheim state intact.
 */
export async function performStatefulHMR(newWasmUrl: string) {
    console.log("[CVKG HMR] Performing stateful hot reload...");
    
    // 1. Capture current Niflheim state from the running app
    const currentState = (window as any).cvkg_capture_state?.();
    
    // 2. Reload the WASM module
    // This assumes the app loader can be re-initialized
    await (window as any).cvkg_reload_wasm?.(newWasmUrl);
    
    // 3. Restore state
    if (currentState) {
        (window as any).cvkg_restore_state?.(currentState);
        console.log("[CVKG HMR] State restored successfully.");
    }
}
