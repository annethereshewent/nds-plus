export class AudioProcessingNode extends AudioWorkletNode {
  constructor(context: AudioContext) {
    super(context, 'audio-processor', {
      numberOfInputs: 1
    })
  }
}