import { WasmEmulator } from "../pkg/ds_emulator_wasm";

const SAMPLE_RATE = 44100
const BUFFER_SIZE = 8192

export class Audio {
  private emulator: WasmEmulator
  constructor(emulator: WasmEmulator) {
    this.emulator = emulator
  }

  startAudio() {
    const audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })

    const scriptProcessor = audioContext.createScriptProcessor(BUFFER_SIZE, 0, 2)

    scriptProcessor.onaudioprocess = (e) => {
      const leftData = e.outputBuffer.getChannelData(0)
      const rightData = e.outputBuffer.getChannelData(1)

      this.emulator.update_audio_buffers(leftData, rightData)
    }

    scriptProcessor.connect(audioContext.destination)
  }
}