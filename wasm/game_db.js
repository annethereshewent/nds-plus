const request = indexedDB.open("ds_saves", 2)


request.onupgradeneeded = (event) => {
  const db = event.target.result

  db.createObjectStore("saves", { keyPath: "gameName" })
}

function getSave(gameName) {

  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ds_saves")

    request.onsuccess = (event) => {
      const db = event.target.result

      const transaction = db.transaction(["saves"], "readwrite")

      const objectStore = transaction.objectStore("saves")

      const request = objectStore.get(gameName)

      request.onsuccess = (event) => {
        resolve(request.result)
      }

      request.onerror = (event) => {
        resolve(null)
      }
    }
  })
}

function deleteDbSave(gameName) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ds_saves")

    request.onsuccess = (event) => {
      const db = event.target.result

      const transaction = db.transaction(["saves"], "readwrite")

      const objectStore = transaction.objectStore("saves")

      const request = objectStore.delete(gameName)

      request.onsuccess = (event) => {
        resolve(true)
      }

      request.onerror = (event) => {
        resolve(false)
      }
    }
  })

}

function updateSave(gameName) {

}

function getUserSaves() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ds_saves")

    request.onsuccess = (event) => {
      const db = event.target.result

      const transaction = db.transaction(["saves"], "readwrite")

      const objectStore = transaction.objectStore("saves")

      const request = objectStore.getAll()

      request.onsuccess = (event) => {
        resolve(request.result)
      }

      request.onerror = (event) => {
        resolve(null)
      }
    }
  })
}

function setSave(gameName, data) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ds_saves")

    request.onsuccess = (event) => {
      const db = event.target.result

      const transaction = db.transaction(["saves"], "readwrite")

      const objectStore = transaction.objectStore("saves")

      const request = objectStore.get(gameName)

      request.onsuccess = (event) => {
        const existing = request.result

        if (existing != null) {
          existing.data = data
          objectStore.put(existing)
        } else {
          objectStore.add({
            gameName,
            data
          })
        }
        resolve(true)
      }

      request.onerror = (event) => {
        const request = objectStore.add({
          gameName,
          data
        })

        request.onsuccess = (event) => {
          resolve(true)
        }
      }
    }
  })

}