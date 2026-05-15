/* tslint:disable */
/* eslint-disable */

/**
 * Applies a sequence of Virtual DOM patches to the browser's actual accessibility DOM.
 *
 * This maintains a parallel tree of hidden ARIA elements corresponding to the
 * drawn visual interface, ensuring accessibility while using Canvas/WebGPU rendering.
 */
export function apply_vdom_patches(serialized_patches: string): void;

/**
 * Get the name of the current rendering tier for display/telemetry
 */
export function get_render_tier_name(): string;

export function wasm_main(): Promise<void>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly wasm_main: () => number;
    readonly apply_vdom_patches: (a: number, b: number, c: number) => void;
    readonly get_render_tier_name: (a: number) => void;
    readonly __wasm_bindgen_func_elem_7739: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_7741: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_3203: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2824: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2824_3: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2824_4: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_4518: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export5: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
