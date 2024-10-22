import { SaveEntry } from "./save_entry"
import { GameStateEntry, StateEntry } from "./game_state_entry"

export class DsDatabase {
  db: IDBDatabase|null = null
  constructor() {
    const request = indexedDB.open("ds_saves", 3)

    request.onsuccess = (event) => {
      this.db = request.result
    }

    request.onupgradeneeded = (event) => {
      this.db = request.result

      // this.db.createObjectStore("saves", { keyPath: "gameName" })
      this.db.createObjectStore("save_states", { keyPath: "gameName" })
    }

    request.onerror = (event) => {
      console.log('an error occurred while retrieving DB')
    }
  }

  getSave(gameName: string): Promise<SaveEntry|null> {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.get(gameName)

      if (request != null) {
        request.onsuccess = (event) => resolve(request.result)
        request.onerror = (event) => resolve(null)
      } else {
        resolve(null)
      }
    })
  }

  async hasSaves(): Promise<boolean> {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.count()

      if (request != null) {
        request.onsuccess = () => resolve(request.result > 0)
        request.onerror = () => resolve(false)
      } else {
        resolve(false)
      }
    })
  }

  setSave(gameName: string, data: Uint8Array) {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.put({
        gameName,
        data
      })

      if (request != null) {
        request.onsuccess = (event) => resolve(true)
        request.onerror = (event) => resolve(false)
      } else {
        resolve(false)
      }
    })
  }

  getObjectStore() {
    const transaction = this.db?.transaction(["saves"], "readwrite")

    const objectStore = transaction?.objectStore("saves")

    return objectStore
  }

  deleteSave(gameName: string) {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.delete(gameName)

      if (request != null) {
        request.onsuccess = (event) => resolve(true)
        request.onerror = (event) => resolve(false)
      } else {
        resolve(false)
      }
    })
  }

  getSaves(): Promise<SaveEntry[]|null> {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.getAll()

      if (request != null) {
        request.onsuccess = (event) => resolve(request.result)
        request.onerror = (event) => resolve(null)
      } else {
        resolve(null)
      }
    })
  }

  getStateObjectStore() {
    const transaction = this.db?.transaction(["save_states"], "readwrite")

    const objectStore = transaction?.objectStore("save_states")

    return objectStore
  }

  createSaveState(gameName: string, data: Uint8Array, stateName: string = "quick_save.state"): Promise<boolean> {
    const objectStore = this.getStateObjectStore()

    const request = objectStore?.get(gameName)

    return new Promise((resolve, reject) => {
      if (request != null) {
        request.onsuccess = (e) => {
          const existing = request.result as GameStateEntry

          if (existing != null) {
            let state = existing.states[stateName]

            if (state == null) {
              existing.states[stateName] = {
                stateName,
                state: data
              }
            } else {
              state.state = data
            }
            objectStore?.put(existing)
            resolve(true)
          } else {
            // create a new state
            const gameStateEntry: GameStateEntry = {
              gameName,
              states: {}
            }

            gameStateEntry.states[stateName] = {
              stateName,
              state: data
            }

            objectStore?.put(gameStateEntry)

            resolve(true)
          }
        }

        request.onerror = () => resolve(false)
      } else {
        resolve(false)
      }
    })

  }

  loadSaveState(gameName: string, stateName: string = "quick_save.state"): Promise<Uint8Array|null> {
    return new Promise((resolve, reject) => {
      const objectStore = this.getStateObjectStore()

      const request = objectStore?.get(gameName)

      if (request != null) {
        request.onsuccess = (e) => {
          const existing = request.result as GameStateEntry

          if (existing != null) {
            const state = existing.states[stateName]

            resolve(state.state)
          } else {
            resolve(null)
          }
        }
        request.onerror = (e) => resolve(null)
      } else {
        resolve(null)
      }
    })
  }
}