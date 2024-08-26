class DSWorkletProcessor extends AudioWorkletProcessor {
  audioBuffer = []

  constructor() {
    super()

    this.port.onmessage = (event) => {
      this.audioBuffer = this.audioBuffer.concat(Array.from(event.data))
    }
  }

  process(inputs, outputs, parameters) {
    const output = outputs[0]

    if (this.audioBuffer.length) {
      let leftIndex = 0
      let rightIndex = 0

      let isLeft = true

      for (let i = 0; i < output[0].length * 2; i++) {
        if (!this.audioBuffer.length) {
          break
        }

        const sample = this.audioBuffer.shift()

        if (isLeft) {
          output[0][leftIndex] = sample
          leftIndex++
        } else {
          output[1][rightIndex] = sample
          rightIndex++
        }

        isLeft = !isLeft
      }
      if (this.audioBuffer.length == 0) {
        this.port.postMessage(true)
      }
    } else {
      this.port.postMessage(true)
    }

    return true
  }
}

registerProcessor('audio-processor', DSWorkletProcessor)