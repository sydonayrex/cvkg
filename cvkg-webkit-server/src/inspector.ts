/**
 * CVKG Agentic Development Guidelines (v1.2) - Guideline 6 Compliance
 * All public Inspector protocol functions have TypeScript JSDoc comments.
 */

/**
 * Represents the layout bounds of a component.
 */
export interface LayoutRect {
    x: number;
    y: number;
    width: number;
    height: number;
}

/**
 * Standard ARIA properties for accessibility mapping.
 */
export interface AriaProps {
    label?: string;
    disabled: boolean;
    hidden: boolean;
}

/**
 * A node in the Virtual DOM tree, matching the Rust `VNode` definition.
 */
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
 * Connects to the CVKG Inspector WebSocket endpoint.
 *
 * @param url - The WebSocket URL of the inspector server (e.g. "ws://localhost:3000/cvkg-ws").
 * @returns A promise that resolves when the WebSocket connection is established.
 */
export function connectInspector(url: string): Promise<WebSocket> {
    return new Promise((resolve, reject) => {
        const ws = new WebSocket(url);
        
        ws.onopen = () => {
            console.log(`[CVKG Inspector] Connected to ${url}`);
            resolve(ws);
        };
        
        ws.onerror = (err) => {
            console.error(`[CVKG Inspector] Connection error:`, err);
            reject(err);
        };
    });
}

/**
 * Requests the latest Virtual DOM snapshot from the running application.
 *
 * @param ws - The active WebSocket connection to the inspector server.
 */
export function requestVDomSnapshot(ws: WebSocket): void {
    const message = {
        type: "request_snapshot",
        payload: {}
    };
    ws.send(JSON.stringify(message));
}

/**
 * Highlights a specific component in the running application by overlaying a bounding box.
 *
 * @param ws - The active WebSocket connection.
 * @param nodeId - The unique NodeId of the component to highlight.
 */
export function highlightComponent(ws: WebSocket, nodeId: number): void {
    const message = {
        type: "highlight_component",
        payload: { id: nodeId }
    };
    ws.send(JSON.stringify(message));
}

/**
 * Mutates the state of a specific component at runtime.
 *
 * @param ws - The active WebSocket connection.
 * @param nodeId - The unique NodeId of the component.
 * @param stateKey - The key of the state property to modify.
 * @param newValue - The new value to inject into the state.
 */
export function overrideComponentState(ws: WebSocket, nodeId: number, stateKey: string, newValue: any): void {
    const message = {
        type: "override_state",
        payload: {
            id: nodeId,
            key: stateKey,
            value: newValue
        }
    };
    ws.send(JSON.stringify(message));
}
