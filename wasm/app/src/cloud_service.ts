import { SaveEntry } from "./save_entry"


const BASE_URL = "https://accounts.google.com/o/oauth2/v2/auth"
const CLIENT_ID = "353451169812-khtanjkfi98eh2bgcldmqt22g47og1ef.apps.googleusercontent.com"

export class CloudService {
  private accessToken: string = ""
  private dsFolderId: string|null = null

  usingCloud = false

  constructor() {
    const signIn = document.getElementById("cloud-button")
    const accessToken = localStorage.getItem("ds_access_token")
    const expiresIn = parseInt(localStorage.getItem("ds_access_expires") || "-1")

    if (signIn != null) {
      if (accessToken == null) {
        signIn.addEventListener("click", () => this.oauthSignIn())
      } else if (expiresIn == -1 || Date.now() / 1000 < expiresIn) {
        this.accessToken = accessToken
        this.usingCloud = true

        signIn.style.display = "none"
        const isLoggedIn = document.getElementById("cloud-logged-in")

        if (isLoggedIn != null) {
          isLoggedIn.style.display = "block"
        }
      } else {
        localStorage.removeItem("ds_access_token")
        localStorage.removeItem("ds_access_expires")

        console.log(`token expired, silently logging in`)

        this.silentSignIn()


        setTimeout(() => {
          this.getTokenFromStorage()

          signIn.style.display = "none"

          const isLoggedIn = document.getElementById("cloud-logged-in")

          if (isLoggedIn != null) {
            isLoggedIn.style.display = "block"
            }
        }, 400)

      }
    }
  }

  async createDsSavesFolder() {
    if (this.dsFolderId == null) {
      const params = new URLSearchParams({
        q: `mimeType = "application/vnd.google-apps.folder" and name="ds-saves"`
      })
      const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

      const json = await this.cloudRequest(() => fetch(url, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`
        },
      }))

      if (json != null && json.files != null && json.files[0] != null) {
        this.dsFolderId = json.files[0].id
      } else {
        // create the folder
        const url = `https://www.googleapis.com/drive/v3/files?uploadType=media`

        const json = await this.cloudRequest(() => fetch(url, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/vnd.google-apps.folder"
          },
          body: JSON.stringify({
            name: "ds-saves",
            mimeType: "application/vnd.google-apps.folder"
          })
        }))


        if (json != null && json.files != null && json.files[0] != null) {
          this.dsFolderId = json.files[0].id
        }
      }
    }
  }

  getTokenFromStorage() {
    const accessToken = localStorage.getItem("ds_access_token")

    if (accessToken != null) {
      this.accessToken = accessToken
      this.usingCloud = true
    }
  }

  async oauthSignIn() {
    const params = await this.getLoginParams()

    console.log(`${BASE_URL}?${params.toString()}`)

    const popup = window.open(`${BASE_URL}?${params.toString()}`, "popup", "popup=true,width=650,height=650,resizable=true")

    if (popup != null) {
      let interval = setInterval(() => {
        if (popup.closed) {
          clearInterval(interval)
          location.reload()
        }
      }, 300)
    }
  }

  silentSignIn() {
    const silentEl = document.getElementById("silent-sign-in") as HTMLIFrameElement

    if (silentEl != null && silentEl.contentWindow != null) {
      const params = this.getLoginParams(true)

      silentEl.contentWindow.window.location.href = `${BASE_URL}?${params.toString()}`
    }
  }

  async cloudRequest(request: () => Promise<Response>, returnBuffer: boolean = false): Promise<any> {
    return new Promise(async (resolve, reject) => {
      const response = await request()

      if (response.status == 200) {
        const data = returnBuffer ? await response.arrayBuffer() : await response.json()

        resolve(data)
      } else if (response.status == 401) {
        this.silentSignIn()

        // allow silent sign in time to do its thing
        setTimeout(async () => {
          this.getTokenFromStorage()

          const response = await request()

          if (response.status == 200) {
            const json = await response.json()
            resolve(json)
          } else {
            resolve(null)
            localStorage.removeItem("ds_access_token")
            localStorage.removeItem("ds_access_expires")
            localStorage.removeItem("ds_user_email")

            this.usingCloud = false
            this.accessToken = ""

          }
        }, 400)
      }

      resolve(null)
    })

  }

  async getLoginParams(noPrompt: boolean = false) {
    const response = await fetch('/env')
    const redirectUri = await response.text()

    console.log(redirectUri.toString())
    const params = new URLSearchParams({
      client_id: CLIENT_ID,
      redirect_uri: redirectUri.toString(),
      response_type: "token",
      scope: "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email",
    })

    if (noPrompt) {
      const email = localStorage.getItem("ds_user_email")

      if (email != null) {
        params.append("prompt", "none")
        params.append("login_hint", email)
      }

    }

    return params
  }

  async getSaveInfo(gameName: string, searchRoot: boolean = false) {
    await this.createDsSavesFolder()

    const fileName = gameName.match(/\.sav$/) ? gameName : `${gameName}.sav`


    const query = searchRoot ? `name = "${fileName}"` : `name = "${fileName}" and parents in "${this.dsFolderId}"`


    const params = new URLSearchParams({
      q: query,
      fields: "files/id,files/parents,files/name"
    })

    const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

    return await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))
  }

  async getSave(gameName: string): Promise<SaveEntry> {
    const json = await this.getSaveInfo(gameName)

    if (json != null && json.files != null) {
      const file = json.files[0]

      if (file != null) {

        // retrieve the file data from the cloud
        const url = `https://www.googleapis.com/drive/v3/files/${file.id}?alt=media`

        const body = await this.cloudRequest(() => fetch(url, {
          headers: {
            Authorization: `Bearer ${this.accessToken}`
          }
        }), true)

        return {
          gameName,
          data: new Uint8Array((body as ArrayBuffer))
        }
      }

    }

    return {
      gameName,
      data: new Uint8Array(0)
    }
  }

  async deleteSave(gameName: string): Promise<boolean> {
    const json = await this.getSaveInfo(gameName)

    if (json != null && json.files != null) {
      const url = `https://www.googleapis.com/drive/v3/files/${json.files[0].id}`

      await this.cloudRequest(() => fetch(url, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`
        },
        method: "DELETE"
      }))

      return true
    }

    return false
  }

  async getSaves(): Promise<SaveEntry[]> {
    await this.createDsSavesFolder()

    const params = new URLSearchParams({
      q: `parents in "${this.dsFolderId}"`
    })
    const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

    const json = await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))

    const saveEntries: SaveEntry[] = []
    if (json != null && json.files != null) {
      for (const file of json.files) {
        saveEntries.push({
          gameName: file.name
        })
      }
    }

    return saveEntries
  }

  async uploadSave(gameName: string, bytes: Uint8Array) {
    const json = await this.getSaveInfo(gameName)

    // this is a hack to get it to change the underlying array buffer
    // (so it doesn't save a bunch of junk from memory unrelated to save)
    const bytesCopy = new Uint8Array(Array.from(bytes))

    const buffer = bytesCopy.buffer

    let resultFile: any
    if (json != null && json.files != null) {
      const file = json.files[0]

      if (file != null) {
        const url = `https://www.googleapis.com/upload/drive/v3/files/${file.id}?uploadType=media`
        await this.cloudRequest(() => fetch(url, {
          method: "PATCH",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/octet-stream",
            "Content-Length": `${bytes.length}`
          },
          body: buffer
        }))
        // there's no need for renaming the file since it's already been uploaded
        return
      } else {
        const url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=media&fields=id,name,parents"
        resultFile = await this.cloudRequest(() => fetch(url, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/octet-stream",
            "Content-Length": `${bytes.length}`
          },
          body: buffer
        }))
      }
    }

    // rename the file to ${gameName}.sav

    if (resultFile != null) {
      let fileName = !gameName.match(/\.sav$/) ? `${gameName}.sav` : gameName

      const params = new URLSearchParams({
        uploadType: "media",
        addParents: this.dsFolderId || "",
        removeParents: resultFile.parents.join(",")
      })

      const url = `https://www.googleapis.com/drive/v3/files/${resultFile.id}?${params.toString()}`

      const json = await this.cloudRequest(() => fetch(url, {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          "Content-Type": "application/octet-stream"
        },
        body: JSON.stringify({
          name: fileName,
          mimeType: "application/octet-stream"
        })
      }))
    }
  }

  async getLoggedInEmail() {
    const url = "https://www.googleapis.com/oauth2/v2/userinfo"

    const json = await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))

    if (json != null && json.email != null) {
      localStorage.setItem("ds_user_email", json.email)
    }
  }

  async checkAuthentication() {
    if (window.location.href.indexOf("#") != -1) {
      const tokenParams = window.location.href.split("#")[1].split("&")

      let accessToken = tokenParams.filter((param) => param.indexOf('access_token') != -1)[0]
      let expires = tokenParams.filter((param) => param.indexOf('expires_in') != -1)[0]

      if (accessToken != null) {
        accessToken = accessToken.split("=")[1]

        if (expires != null) {
          expires = expires.split("=")[1]

          const timestamp = parseInt(expires) + Date.now() / 1000

          localStorage.setItem("ds_access_expires", timestamp.toString())
        }

        localStorage.setItem("ds_access_token", accessToken)

        this.accessToken = accessToken
        this.usingCloud = true

        // finally get logged in user email
        await this.getLoggedInEmail()

        window.close()
      }
    }

  }
}