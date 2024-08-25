class AudioProcessingNode extends AudioWorkletNode {
  constructor(context) {
    super(context, 'audio-processor', {
      numberOfOutputs : 2,
      outputChannelCount : [2, 2]
    })
  }

  updateBuffer(buffer) {
    this.port.postMessage(buffer)
  }
}