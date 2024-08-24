const request = indexedDB.open("ds_saves", 2)


request.onupgradeneeded = (event) => {
  console.log('test????')
  const db = event.target.result

  console.log(db)

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

function setSave(gameName, data) {
  console.log('im being called a bunch somehow')
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ds_saves")

    request.onsuccess = (event) => {
      const db = event.target.result

      const transaction = db.transaction(["saves"], "readwrite")

      const objectStore = transaction.objectStore("saves")

      const request = objectStore.get(gameName)

      request.onsuccess = (event) => {
        console.log(request)
        const existing = request.result

        if (existing != null) {
          existing.data = data
          objectStore.put(existing)
        } else {
          console.log('storing save in database')
          objectStore.add({
            gameName,
            data
          })
        }
        resolve(true)
      }

      request.onerror = (event) => {
        console.log('record not found, creating')

        const request = objectStore.add({
          gameName,
          data
        })

        request.onsuccess = (event) => {
          console.log('successfully added save to db')
          resolve(true)
        }
      }
    }
  })

}