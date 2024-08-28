import { SaveEntry } from "./ds_database"

const BASE_URL = "https://accounts.google.com/o/oauth2/v2/auth"
const CLIENT_ID = "353451169812-khtanjkfi98eh2bgcldmqt22g47og1ef.apps.googleusercontent.com"

export class CloudService {
  private accessToken: string = ""

  usingCloud = false

  constructor() {
    const signIn = document.getElementById("cloud-button")
    const accessToken = localStorage.getItem("ds_access_token")

    if (signIn != null) {
      if (accessToken == null) {
        signIn.addEventListener("click", () => this.oauthSignIn())
      } else {
        this.accessToken = accessToken
        this.usingCloud = true

        signIn.style.display = "none"
        const isLoggedIn = document.getElementById("cloud-logged-in")

        if (isLoggedIn != null) {
          isLoggedIn.style.display = "block"
        }
      }
    }
  }

  oauthSignIn() {
    const params = new URLSearchParams({
      client_id: CLIENT_ID,
      redirect_uri: "http://localhost:8080",
      response_type: "token",
      scope: "https://www.googleapis.com/auth/drive.file"
    })

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

  getSave(gameName: string): SaveEntry {
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

    console.log(json)

    return []
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
      console.log(file.id)
      const url = `https://www.googleapis.com/upload/drive/v3/files/${file.id}?uploadType=media`

      const response = await fetch(url, {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
        },
        body: JSON.stringify({
          name: `${gameName}.sav`,
          mimeType: "application/octet-stream"
        })
      })

      const json = await response.json()

      console.log(`got past update so it should have renamed the file. (PS FUCK YOU GOOGLE I HATE YOU)`)

      console.log(json)
    }
  }

  checkAuthentication() {
    if (window.location.href.indexOf("#") != -1) {
      const tokenParams = window.location.href.split("#")[1].split("&")

      let accessToken = tokenParams.filter((param) => param.indexOf('access_token') != -1)[0]
      let refreshToken = tokenParams.filter((param) => param.indexOf('refresh_token') != -1)[0]

      if (accessToken != null) {
        accessToken = accessToken.split("=")[1]

        if (refreshToken != null) {
          refreshToken = refreshToken.split("=")[1]
          localStorage.setItem("ds_refresh_token", refreshToken)
        }

        localStorage.setItem("ds_access_token", accessToken)

        window.close()
      }
    }

  }
}