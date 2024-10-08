import { WasmEmulator } from "../../pkg/ds_emulator_wasm";
import { AudioProcessingNode } from "./audio_processing_node";

const SAMPLE_RATE = 44100
const BUFFER_SIZE = 8192

const NUM_SAMPLES = 2048

export class Audio {
  private emulator: WasmEmulator
  private audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })
  private scriptProcessor: ScriptProcessorNode = this.audioContext.createScriptProcessor(BUFFER_SIZE, 0, 2)
  private node: AudioProcessingNode|null = null

  private samples: number[] = []
  private sampleIndex = 0

  private mediaRecorder: MediaRecorder|null = null

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

  async startMicrophone() {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })

    const micNode = this.audioContext.createMediaStreamSource(stream)

    await this.audioContext.audioWorklet.addModule("processors.js")

    this.node = new AudioProcessingNode(this.audioContext)

    micNode.connect(this.node)

    this.node.port.onmessage = (e) => {
      const newSamples = Array.from(e.data) as number[]

      if (newSamples.length + this.sampleIndex >= NUM_SAMPLES) {
        this.sampleIndex = 0
      }

      let i = 0
      while (i < newSamples.length && this.sampleIndex < NUM_SAMPLES) {
        this.samples[this.sampleIndex] = newSamples[i];

        i++
        this.sampleIndex++
      }
    }
  }

  updateMicBuffer() {
   this. emulator.update_mic_buffer(new Float32Array(this.samples))
  }

  stopAudio() {
    this.scriptProcessor.disconnect(this.audioContext.destination)
    this.scriptProcessor.onaudioprocess = null
  }
}