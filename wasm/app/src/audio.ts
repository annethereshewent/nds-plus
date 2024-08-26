import { InitOutput, WasmEmulator } from "../../pkg/ds_emulator_wasm";
import { AudioProcessingNode } from "./audio_processing_node";

const SAMPLE_RATE = 44100

export class Audio {
  private emulator: WasmEmulator
  node: AudioProcessingNode|null = null
  private wasm: InitOutput

  constructor(emulator: WasmEmulator, wasm: InitOutput) {
    this.emulator = emulator
    this.wasm = wasm
  }

  async startAudio() {
    const audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })

    await audioContext.resume()

    audioContext.audioWorklet.addModule("processors.js").then(() => {
      this.node = new AudioProcessingNode(audioContext)

      const audioBuffer = new Float32Array(this.wasm.memory.buffer, this.emulator.get_audio_buffer(), this.emulator.get_buffer_length())
      this.node.updateBuffer(audioBuffer)

      this.node.connect(audioContext.destination)
    })

    audioContext.resume()
  }

  updateAudioBuffer() {
    const bufferLength = this.emulator.get_buffer_length()

    const audioBuffer = new Float32Array(this.wasm.memory.buffer, this.emulator.get_audio_buffer(), bufferLength)
    this.node?.updateBuffer(audioBuffer)
  }

}