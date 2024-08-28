import { SaveEntry } from "./save_entry"


const BASE_URL = "https://accounts.google.com/o/oauth2/v2/auth"
const CLIENT_ID = "353451169812-khtanjkfi98eh2bgcldmqt22g47og1ef.apps.googleusercontent.com"

export class CloudService {
  private accessToken: string = ""

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
      }
    }
  }

  oauthSignIn() {
    const params = this.getLoginParams()

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

  getLoginParams(noPrompt: boolean = false) {
    const params = new URLSearchParams({
      client_id: CLIENT_ID,
      redirect_uri: "http://localhost:8080",
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

  async getSave(gameName: string): Promise<SaveEntry> {
    const params = new URLSearchParams({
      q: `name = ${gameName}.sav`
    })

    const url = `https://www.googleapis.com/drive/v3/files?q=${params.toString()}`

    const response = await fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    })

    console.log(response)

    return {
      gameName: "",
      data: new Uint8Array(0)
    }
  }

  async getSaves(): Promise<SaveEntry[]> {
    const url = "https://www.googleapis.com/drive/v3/files"

    const response = await fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    })

    const json = await response.json()

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
    const url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=media"

    const buffer = bytes.buffer

    const response = await fetch(url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.accessToken}`,
        "Content-Type": "application/octet-stream",
        "Content-Length": `${bytes.length}`
      },
      body: buffer
    })

    const file = await response.json()

    // rename the file to ${gameName}.sav

    if (file != null) {
      const url = `https://www.googleapis.com/drive/v3/files/${file.id}?uploadType=media`

      const response = await fetch(url, {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          "Content-Type": "application/octet-stream"
        },
        body: JSON.stringify({
          name: `${gameName}.sav`,
          mimeType: "application/octet-stream"
        })
      })

      const json = await response.json()

      console.log(json)
    }
  }

  async getLoggedInEmail() {
    const url = "https://www.googleapis.com/oauth2/v2/userinfo"

    const response = await fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    })

    const json = await response.json()

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

          console.log(expires)

          const timestamp = parseInt(expires) + Date.now() / 1000

          localStorage.setItem("ds_access_expires", timestamp.toString())
        }

        localStorage.setItem("ds_access_token", accessToken)

        this.accessToken = accessToken

        // finally get logged in user email
        await this.getLoggedInEmail()

        window.close()
      }
    }

  }
}