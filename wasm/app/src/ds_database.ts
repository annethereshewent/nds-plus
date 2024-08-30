import { SaveEntry } from "./save_entry"

export class DsDatabase {
  db: IDBDatabase|null = null
  constructor() {
    const request = indexedDB.open("ds_saves", 2)

    request.onsuccess = (event) => {
      this.db = request.result
    }

    request.onupgradeneeded = (event) => {
      this.db = request.result

      this.db.createObjectStore("saves", { keyPath: "gameName" })
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

      const request = objectStore?.get(gameName)

      if (request != null) {
        request.onsuccess = (event) => {
          const existing = request.result

          if (existing != null) {
            existing.data = data

            const request = objectStore?.put(existing)

            if (request != null) {
              request.onsuccess = (event) => resolve(true)
              request.onerror = (event) => resolve(false)
            } else {
              resolve(false)
            }
          } else {
            const saveEntry: SaveEntry = {
              gameName,
              data
            }
            const request = objectStore?.add(saveEntry)

            if (request != null) {
              request.onsuccess = (event) => resolve(true)
              request.onerror = (event) => resolve(false)
            }
          }
        }
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
}