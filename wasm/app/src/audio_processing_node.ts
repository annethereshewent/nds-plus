export class AudioProcessingNode extends AudioWorkletNode {

  bufferReady = true

  constructor(context: AudioContext) {
    super(context, 'audio-processor', {
      numberOfOutputs : 2,
      outputChannelCount : [2, 2]
    })

    this.port.onmessage = (event) => {
      this.bufferReady = event.data
    }
  }

  updateBuffer(buffer: Float32Array) {
    this.port.postMessage(buffer)
    this.bufferReady = false
  }
}

// audio_processing_node.ts