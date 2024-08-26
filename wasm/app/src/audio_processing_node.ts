export class AudioProcessingNode extends AudioWorkletNode {
  constructor(context: AudioContext) {
    super(context, 'audio-processor', {
      numberOfOutputs : 2,
      outputChannelCount : [2, 2]
    })
  }

  updateBuffer(buffer: Float32Array) {
    this.port.postMessage(buffer)
  }
}