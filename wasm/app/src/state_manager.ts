
import { InitOutput, WasmEmulator } from '../../pkg/ds_emulator_wasm'
import { StateDatabase } from './state_database'
import { zlib, unzlib } from 'fflate'

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

      return new Promise((resolve, reject) => {
        zlib(data, { level: 2 }, async (err, compressed) => {
          await this.db.createSaveState(this.gameName, compressed)
          resolve(true)
        })
      })

    }
  }

  async loadSaveState(): Promise<Uint8Array|null> {
    const compressed = await this.db.loadSaveState(this.gameName)

    return new Promise((resolve, reject) => {
      if (compressed != null) {
        unzlib(compressed, (err, data) => {
          resolve(data)
        })
      } else {
        resolve(null)
      }
    })

  }
}