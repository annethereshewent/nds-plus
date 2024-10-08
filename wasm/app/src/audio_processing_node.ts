export class AudioProcessingNode extends AudioWorkletNode {

  bufferReady = true

  constructor(context: AudioContext) {
    super(context, 'audio-processor', {
      numberOfInputs: 1
    })
  }
}