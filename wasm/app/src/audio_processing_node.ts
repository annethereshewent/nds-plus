export class AudioProcessingNode extends AudioWorkletNode {

  bufferReady = true

  constructor(context: AudioContext) {
    super(context, 'audio-processor', {
      numberOfOutputs : 2,
      outputChannelCount : [2, 2]
    })
  }
}