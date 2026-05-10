/**
 * CVKG Agentic Development Guidelines (v1.2) — Guideline 6 Compliance
 * All public Inspector protocol functions carry TypeScript JSDoc comments.
 */

// ─── Shared Types ──────────────────────────────────────────────────────────────

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
    props: Record<string, unknown>;
    state?: Record<string, unknown>;
    layout: LayoutRect;
    children: number[];
    aria_role: string;
    aria_props: AriaProps;
}

/**
 * The four rendering backends the CVKG server may negotiate.
 * Must stay in sync with the Rust `RenderBackend` enum in main.rs.
 */
export type RenderBackend = "native" | "wgpu" | "webgl2" | "wasm";

// ─── Capability Detection ─────────────────────────────────────────────────────

/**
 * Detects the highest-capability rendering backend available in the current
 * browser context and returns the full capability string sent to the server
 * via the `X-CVKG-Caps` request header.
 *
 * This is the client-side Phase-1 probe.  It runs synchronously for the
 * WebGL2 and WASM checks; the WebGPU check is async because adapter
 * availability requires a GPU request round-trip.
 *
 * Contract: callers MUST `await` this before making any fetch that should
 * carry the `X-CVKG-Caps` header, including the HMR WebSocket upgrade.
 */
export async function detectCapabilities(): Promise<string> {
    const caps: string[] = ["wasm"]; // guaranteed baseline

    // WebGL2 — synchronous context probe.
    try {
        const canvas = document.createElement("canvas");
        if (canvas.getContext("webgl2")) caps.push("webgl2");
    } catch {
        // Older browsers may throw on getContext; treat as unavailable.
    }

    // WebGPU — async adapter request required.
    // `navigator.gpu` exists in Chrome 113+, Edge 113+, and Firefox Nightly.
    if ("gpu" in navigator) {
        try {
            const adapter = await (navigator as unknown as { gpu: GPU }).gpu.requestAdapter();
            if (adapter) caps.push("webgpu");
        } catch {
            // GPU blocked (e.g. headless, hardware acceleration disabled).
        }
    }

    // Native wgpu — only valid inside a wry WebView that injects this global.
    if (typeof (window as unknown as { __CVKG_NATIVE__?: boolean }).__CVKG_NATIVE__ === "boolean") {
        caps.push("native");
    }

    const capsString = caps.join(",");
    sessionStorage.setItem("cvkg_caps", capsString);
    window.__CVKG_CAPS__ = capsString;
    return capsString;
}

// ─── Rendering Backend Bridge ─────────────────────────────────────────────────

/**
 * Unified interface that every backend-specific renderer must satisfy.
 *
 * The bridge dispatches draw calls to whichever implementation is active,
 * allowing the Inspector and HMR code to be backend-agnostic.
 */
export interface RenderBackendAdapter {
    /** Human-readable label for diagnostics (e.g. "WebGPU / WGSL"). */
    readonly label: string;
    /** The backend identifier. */
    readonly backend: RenderBackend;

    /**
     * Draws a single highlighted VNode onto whatever surface this backend manages.
     * For canvas-backed backends this writes directly to a 2D context;
     * for WebGPU this issues a render pass; for WASM it calls into the Rust renderer.
     */
    drawHighlight(node: VNode, fill: string, stroke: string): void;

    /** Clears the entire rendering surface. */
    clear(): void;

    /** Called whenever the viewport is resized. */
    resize(width: number, height: number): void;
}

/**
 * Canvas 2D adapter — shared by the WebGL2 and WASM backends for the
 * inspector overlay.  (WebGL2 and WASM rendering happens inside the WASM
 * module itself; the overlay uses a lightweight 2D canvas so it does not
 * interfere with the WASM surface.)
 */
class Canvas2DAdapter implements RenderBackendAdapter {
    readonly label: string;
    readonly backend: RenderBackend;

    private ctx: CanvasRenderingContext2D;

    constructor(ctx: CanvasRenderingContext2D, backend: RenderBackend, label: string) {
        this.ctx = ctx;
        this.backend = backend;
        this.label = label;
    }

    drawHighlight(node: VNode, fill: string, stroke: string): void {
        const { x, y, width, height } = node.layout;
        this.ctx.fillStyle = fill;
        this.ctx.strokeStyle = stroke;
        this.ctx.lineWidth = 2;
        this.ctx.fillRect(x, y, width, height);
        this.ctx.strokeRect(x, y, width, height);

        // Component type + id label — positioned above the box when space allows.
        this.ctx.fillStyle = "white";
        this.ctx.font = "12px monospace";
        this.ctx.fillText(`${node.component_type} [${node.id}]`, x + 2, Math.max(y - 4, 14));
    }

    clear(): void {
        this.ctx.clearRect(0, 0, this.ctx.canvas.width, this.ctx.canvas.height);
    }

    resize(width: number, height: number): void {
        this.ctx.canvas.width = width;
        this.ctx.canvas.height = height;
    }
}


/**
 * Factory: returns the correct `RenderBackendAdapter` for the negotiated backend.
 *
 * For canvas-backed backends (WebGL2, WASM) the adapter wraps the supplied 2D
 * context.  For WebGPU the caller should call `adapter.init(device, context)`
 * once the WASM module provides those handles.
 */
export function createAdapter(
    backend: RenderBackend,
    overlayCtx: CanvasRenderingContext2D,
): RenderBackendAdapter {
    switch (backend) {
    case "native":
    case "wgpu":
    case "webgl2":
    case "wasm":
        return new Canvas2DAdapter(overlayCtx, backend, backend.toUpperCase());
    }
}

// ─── Inspector Overlay ────────────────────────────────────────────────────────

/**
 * High-performance Inspector Overlay for real-time UI debugging.
 *
 * Renders highlighted bounding boxes over VNodes using whichever
 * `RenderBackendAdapter` matches the active rendering backend.
 * The overlay canvas sits at z-index 9999 with pointer-events disabled
 * so it never intercepts user interaction.
 */
export class InspectorOverlay {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private nodes: Map<number, VNode> = new Map();
    private hoveredId: number | null = null;
    private adapter: RenderBackendAdapter;

    constructor(backend: RenderBackend = window.__CVKG_BACKEND__ ?? "wasm") {
        this.canvas = document.createElement("canvas");
        Object.assign(this.canvas.style, {
            position: "fixed",
            top: "0",
            left: "0",
            pointerEvents: "none",
            zIndex: "9999",
        });
        document.body.appendChild(this.canvas);

        const ctx = this.canvas.getContext("2d");
        if (!ctx) throw new Error("[CVKG Inspector] Failed to obtain 2D overlay context");
        this.ctx = ctx;

        this.adapter = createAdapter(backend, ctx);

        this.resize();
        window.addEventListener("resize", () => this.resize());

        console.log(`[CVKG Inspector] Overlay active — backend: ${this.adapter.label}`);
    }


    private resize(): void {
        this.adapter.resize(window.innerWidth, window.innerHeight);
    }

    /**
     * Replaces the local VNode cache with a fresh snapshot and re-renders.
     *
     * Called by the WASM runtime after every commit phase so the overlay
     * stays in sync with the virtual DOM tree.
     */
    public updateSnapshot(nodes: VNode[]): void {
        this.nodes.clear();
        for (const n of nodes) this.nodes.set(n.id, n);
        this.render();
    }

    /**
     * Highlights the node with the given `id`, or clears all highlights when
     * `id` is `null`.  Intended to be called from `pointermove` handlers or
     * devtools panel selection events.
     */
    public highlight(id: number | null): void {
        this.hoveredId = id;
        this.render();
    }

    private render(): void {
        this.adapter.clear();
        if (this.hoveredId === null) return;

        const node = this.nodes.get(this.hoveredId);
        if (node) {
            this.adapter.drawHighlight(node, "rgba(0,120,255,0.25)", "rgba(0,120,255,0.85)");
        }
    }
}

// ─── State-Preserving HMR ────────────────────────────────────────────────────

/**
 * Performs a stateful hot module replacement for the given backend.
 *
 * Protocol:
 * 1. Capture the current Niflheim state via the WASM-exported `cvkg_capture_state`.
 * 2. Request the new WASM module URL — backend-specific so the browser cache
 *    can hold multiple backends without eviction.
 * 3. Re-initialise the WASM loader for the same backend.
 * 4. Restore state via `cvkg_restore_state`.
 *
 * The backend parameter ensures we reload the correct `pkg/<backend>/` bundle
 * and do not accidentally swap to a different renderer mid-session.
 */
export async function performStatefulHMR(
    newWasmUrl: string,
    backend: RenderBackend = window.__CVKG_BACKEND__ ?? "wasm",
): Promise<void> {
    console.log(`[CVKG HMR] Hot reload — backend: ${backend}, url: ${newWasmUrl}`);

    const cvkg = window as unknown as CvkgRuntime;

    // 1. Capture running state.
    const state = cvkg.cvkg_capture_state?.();

    // 2. Reload the WASM module for this backend.
    await cvkg.cvkg_reload_wasm?.(newWasmUrl, backend);

    // 3. Restore state so component trees survive the swap.
    if (state !== undefined) {
        cvkg.cvkg_restore_state?.(state);
        console.log("[CVKG HMR] State restored.");
    } else {
        console.warn("[CVKG HMR] No state captured — app will cold-start.");
    }
}

// ─── Global Augmentation ─────────────────────────────────────────────────────

/**
 * WASM-exported runtime functions injected into `window` by the CVKG WASM module.
 * Typed here so TypeScript callers don't have to cast to `any`.
 */
interface CvkgRuntime {
    cvkg_capture_state?: () => unknown;
    cvkg_restore_state?: (state: unknown) => void;
    cvkg_reload_wasm?: (url: string, backend: RenderBackend) => Promise<void>;
}

declare global {
    interface Window {
        /** Comma-separated capability tokens, e.g. `"wasm,webgl2,webgpu"`. */
        __CVKG_CAPS__?: string;
        /** The backend negotiated by the server for this session. */
        __CVKG_BACKEND__?: RenderBackend;
        /** Present only inside a wry WebView with direct GPU access. */
        __CVKG_NATIVE__?: boolean;
    }

    /** WebGPU types — present when `navigator.gpu` is available. */
    interface GPU {
        requestAdapter(options?: GPURequestAdapterOptions): Promise<GPUAdapter | null>;
    }
    interface GPUAdapter {}
    interface GPUDevice {}
    interface GPUCanvasContext {}
    interface GPURequestAdapterOptions {}
}
