import { WasmEmulator } from "../../pkg/ds_emulator_wasm";

const SAMPLE_RATE = 44100
const BUFFER_SIZE = 8192

export class Audio {
  private emulator: WasmEmulator
  private audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })
  private scriptProcessor: ScriptProcessorNode = this.audioContext.createScriptProcessor(BUFFER_SIZE, 0, 2)

  constructor(emulator: WasmEmulator) {
    this.emulator = emulator
  }

  startAudio() {
    this.scriptProcessor.onaudioprocess = (e) => {
      const leftData = e.outputBuffer.getChannelData(0)
      const rightData = e.outputBuffer.getChannelData(1)

      this.emulator.update_audio_buffers(leftData, rightData)
    }

    this.scriptProcessor.connect(this.audioContext.destination)
  }

  stopAudio() {
    this.scriptProcessor.disconnect(this.audioContext.destination)
    this.scriptProcessor.onaudioprocess = null
  }
}