
import { InitOutput, WasmEmulator } from '../../pkg/ds_emulator_wasm'
import { StateDatabase } from './state_database'
import { zlibSync, unzlibSync } from 'fflate'

export class StateManager {
  emulator: WasmEmulator
  wasm: InitOutput|null = null
  db = new StateDatabase()
  gameName: string

  constructor(emulator: WasmEmulator, wasm: InitOutput|null, gameName: string) {
    this.emulator = emulator
    this.wasm = wasm
    this.gameName = gameName
  }

  async createSaveState() {
    if (this.wasm != null) {
      const data = new Uint8Array(this.wasm.memory.buffer, this.emulator.create_save_state(), this.emulator.save_state_length())

      const compressed = zlibSync(data, { level: 9 })

      this.db.createSaveState(this.gameName, compressed)
    }
  }

  async loadSaveState(): Promise<Uint8Array|null> {
    const compressed = await this.db.loadSaveState(this.gameName)

    if (compressed != null) {
      const data = unzlibSync(compressed)

      return data
    }

    return null
  }
}