
import { InitOutput, WasmEmulator } from '../../pkg/ds_emulator_wasm'
import { DsDatabase } from './ds_database'
import { zlib, unzlib } from 'fflate'
import { GameStateEntry } from './game_state_entry'

export class StateManager {
  emulator: WasmEmulator
  wasm: InitOutput|null = null
  db: DsDatabase
  gameName: string

  constructor(
    emulator: WasmEmulator,
    wasm: InitOutput|null,
    gameName: string,
    db: DsDatabase
  ) {
    this.emulator = emulator
    this.wasm = wasm
    this.gameName = gameName
    this.db = db
  }

  async createSaveState(stateName = "quick_save.state"): Promise<GameStateEntry|null> {
    if (this.wasm != null) {
      const data = new Uint8Array(this.wasm.memory.buffer, this.emulator.create_save_state(), this.emulator.save_state_length())

      return new Promise((resolve, reject) => {
        zlib(data, { level: 2 }, async (err, compressed) => {
          const entry = await this.db.createSaveState(this.gameName, compressed, stateName)
          resolve(entry)
        })
      })
    }

    return null
  }

  async decompress(compressed: Uint8Array): Promise<Uint8Array> {
    return new Promise((resolve, reject) => {
      unzlib(compressed, (err, data) => {
        resolve(data)
      })
    })
  }

  async loadSaveState(stateName = "quick_save.state"): Promise<Uint8Array|null> {
    const compressed = await this.db.loadSaveState(this.gameName, stateName)

    return new Promise(async (resolve, reject) => {
      if (compressed != null) {
        const data = await this.decompress(compressed)
        resolve(data)
      } else {
        resolve(null)
      }
    })
  }
}