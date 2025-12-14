/* tslint:disable */
/* eslint-disable */

/**
 * Compression levels for PDF optimization
 */
export enum CompressionLevel {
  Light = 0,
  Medium = 1,
  High = 2,
}

/**
 * Convert string to compression level
 */
export function compression_level_from_string(level: string): CompressionLevel;

/**
 * Extract JPEG images from PDF for parallel worker compression
 */
export function extract_images(pdf_data: Uint8Array): any;

/**
 * Get PDF info without compression
 */
export function get_pdf_info(pdf_data: Uint8Array): any;

export function get_version(): string;

/**
 * Initialize panic hook for better WASM error messages
 */
export function init_panic_hook(): void;

/**
 * Reinject compressed images back into PDF preserving ALL metadata and references
 */
export function reinject_images(pdf_data: Uint8Array, compressed_images: any): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init_panic_hook: () => void;
  readonly compression_level_from_string: (a: number, b: number) => [number, number, number];
  readonly get_pdf_info: (a: number, b: number) => [number, number, number];
  readonly extract_images: (a: number, b: number) => [number, number, number];
  readonly reinject_images: (a: number, b: number, c: any) => [number, number, number, number];
  readonly get_version: () => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
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
