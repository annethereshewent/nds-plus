interface GameEntry {
  gameName: string
  states: StateEntry[]
}

interface StateEntry {
  stateName: string,
  state: Uint8Array
}

export class StateDatabase {
  db: IDBDatabase|null = null
  constructor() {
    const request = indexedDB.open("ds_save_states", 2)

    request.onsuccess = (event) => {
      this.db = request.result
    }

    request.onupgradeneeded = (event) => {
      console.log("hopefully this fires")
      this.db = request.result

      this.db.createObjectStore("save_states", { keyPath: "gameName" })
    }

    request.onerror = (event) => {
      console.log('an error occurred while retrieving DB')
    }
  }

  getObjectStore() {
    const transaction = this.db?.transaction(["save_states"], "readwrite")

    const objectStore = transaction?.objectStore("save_states")

    return objectStore
  }

  createSaveState(gameName: string, data: Uint8Array): Promise<boolean> {
    const objectStore = this.getObjectStore()

    const request = objectStore?.get(gameName)

    return new Promise((resolve, reject) => {
      if (request != null) {
        request.onsuccess = (e) => {
          const existing = request.result as GameEntry

          if (existing != null) {
            for (let state of existing.states) {
              if (state.stateName == "save_1.state") {
                // TODO: support multiple save states
                state.state = data

                objectStore?.put(existing)
                resolve(true)
                break
              }
            }
          } else {
            // create a new state
            objectStore?.put({
              gameName,
              states: [{
                stateName: "save_1.state",
                states: [data]
              }]
            })

            resolve(true)
          }
        }

        request.onerror = () => resolve(false)
      } else {
        resolve(false)
      }
    })

  }

  loadSaveState(gameName: string): Promise<Uint8Array|null> {
    return new Promise((resolve, reject) => {
      const objectStore = this.getObjectStore()

      const request = objectStore?.get(gameName)

      if (request != null) {
        request.onsuccess = (e) => {
          const existing = request.result as GameEntry

          if (existing != null) {
            let foundState = false
            for (let state of existing.states) {
              if (state.stateName == "save_1.state") {
                // TODO: support multiple save states
                resolve(state.state)
                break
              }
            }
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