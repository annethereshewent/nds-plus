export interface GameStateEntry {
  gameName: string
  states: { [stateName: string]: StateEntry }
}

export interface StateEntry {
  stateName: string,
  state: Uint8Array,
  imageUrl: string
}