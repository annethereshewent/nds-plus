/* tslint:disable */
/* eslint-disable */
/**
*/
export enum ButtonEvent {
  ButtonA = 0,
  ButtonB = 1,
  ButtonY = 2,
  ButtonX = 3,
  ButtonL = 4,
  ButtonR = 5,
  Select = 6,
  Start = 7,
  Up = 8,
  Down = 9,
  Left = 10,
  Right = 11,
  ButtonR3 = 12,
}
/**
*/
export class WasmEmulator {
  free(): void;
/**
* @param {Uint8Array} bios7_bytes
* @param {Uint8Array} bios9_bytes
* @param {Uint8Array} firmware_bytes
* @param {Uint8Array} game_data
*/
  constructor(bios7_bytes: Uint8Array, bios9_bytes: Uint8Array, firmware_bytes: Uint8Array, game_data: Uint8Array);
/**
* @param {number} x
* @param {number} y
*/
  touch_screen(x: number, y: number): void;
/**
*/
  release_screen(): void;
/**
* @returns {number}
*/
  get_game_code(): number;
/**
* @returns {boolean}
*/
  has_saved(): boolean;
/**
* @returns {number}
*/
  backup_pointer(): number;
/**
* @returns {number}
*/
  backup_length(): number;
/**
* @param {ButtonEvent} button_event
* @param {boolean} value
*/
  update_input(button_event: ButtonEvent, value: boolean): void;
/**
* @param {boolean} val
*/
  set_saved(val: boolean): void;
/**
* @param {string} save_type
* @param {number} ram_capacity
* @param {Uint8Array} bytes
*/
  set_backup(save_type: string, ram_capacity: number, bytes: Uint8Array): void;
/**
* @param {Float32Array} left_buffer
* @param {Float32Array} right_buffer
*/
  update_audio_buffers(left_buffer: Float32Array, right_buffer: Float32Array): void;
/**
* @returns {number}
*/
  get_engine_a_picture_pointer(): number;
/**
* @returns {number}
*/
  get_engine_b_picture_pointer(): number;
/**
* @returns {boolean}
*/
  is_top_a(): boolean;
/**
*/
  press_screen(): void;
/**
* @param {number} x
* @param {number} y
*/
  touch_screen_controller(x: number, y: number): void;
/**
*/
  step_frame(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmemulator_free: (a: number, b: number) => void;
  readonly wasmemulator_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly wasmemulator_touch_screen: (a: number, b: number, c: number) => void;
  readonly wasmemulator_release_screen: (a: number) => void;
  readonly wasmemulator_get_game_code: (a: number) => number;
  readonly wasmemulator_has_saved: (a: number) => number;
  readonly wasmemulator_backup_pointer: (a: number) => number;
  readonly wasmemulator_backup_length: (a: number) => number;
  readonly wasmemulator_update_input: (a: number, b: number, c: number) => void;
  readonly wasmemulator_set_saved: (a: number, b: number) => void;
  readonly wasmemulator_set_backup: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasmemulator_update_audio_buffers: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmemulator_get_engine_a_picture_pointer: (a: number) => number;
  readonly wasmemulator_get_engine_b_picture_pointer: (a: number) => number;
  readonly wasmemulator_is_top_a: (a: number) => number;
  readonly wasmemulator_press_screen: (a: number) => void;
  readonly wasmemulator_touch_screen_controller: (a: number, b: number, c: number) => void;
  readonly wasmemulator_step_frame: (a: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
