
import { InitOutput, WasmEmulator } from '../../pkg/ds_emulator_wasm'
import { StateDatabase } from './state_database'

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

      const stream = new Blob([data]).stream()

      const compressedReadableStream = stream.pipeThrough(
        new CompressionStream("gzip")
      )

      const reader = compressedReadableStream.getReader()

      let compressedData = new Uint8Array()

      let result: ReadableStreamReadResult<Uint8Array>
      while ((result = await reader.read())) {
        if (result.done) {
          break
        } else {
          compressedData = new Uint8Array([...compressedData, ...result.value])
        }
      }

      this.db.createSaveState(this.gameName, compressedData)
    }
  }

  async loadSaveState(): Promise<Uint8Array|null> {
    const compressed = await this.db.loadSaveState(this.gameName)

    if (compressed != null) {
      const stream = new Blob([compressed]).stream()

      const decompressedReadableStream = stream.pipeThrough(
        new DecompressionStream('gzip')
      )

      const reader = decompressedReadableStream.getReader()

      let data = new Uint8Array()

      let result: ReadableStreamReadResult<Uint8Array>
      while ((result = await reader.read())) {
        if (result.done) {
          break
        } else {
          data = new Uint8Array([...data, ...result.value])
        }
      }

      return data
    }

    return null
  }
}