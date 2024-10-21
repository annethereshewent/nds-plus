
import { fs } from 'memfs'
import { ZstdInit } from '@oneidentity/zstd-js'
import { InitOutput, WasmEmulator } from '../../pkg/ds_emulator_wasm'

export class StateManager {
  emulator: WasmEmulator
  wasm: InitOutput|null = null

  constructor(emulator: WasmEmulator, wasm: InitOutput|null) {
    this.emulator = emulator
    this.wasm = wasm
  }

  async createSaveState() {
    if (this.wasm != null) {
      const data = new Uint8Array(this.wasm.memory.buffer, this.emulator.create_save_state(), this.emulator.save_state_length())

      const { ZstdSimple } = await ZstdInit()

      const compressed: Uint8Array = ZstdSimple.compress(data)

      // save data to disk!
      await fs.promises.writeFile("state_1.sav", compressed)
    }
  }

  async loadSaveState() {
    const compressed = await fs.promises.readFile("state_1.sav") as Uint8Array

    const { ZstdSimple } = await ZstdInit()

    const data = ZstdSimple.decompress(compressed)

    this.emulator.load_save_state(data)
  }
}