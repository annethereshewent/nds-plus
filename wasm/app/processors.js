class DSWorkletProcessor extends AudioWorkletProcessor {
  audioBuffer = []

  constructor() {
    super()

    this.port.onmessage = (event) => {
      this.audioBuffer = this.audioBuffer.concat(Array.from(event.data))
    }
  }

  process(inputs, outputs, parameters) {
    const input = inputs[0]

    const samples = input[0]

    this.port.postMessage(samples)

    return true
  }
}

registerProcessor('audio-processor', DSWorkletProcessor)

// processors.js