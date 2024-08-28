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

  checkAuthentication() {
    if (window.location.href.indexOf("#") != -1) {
      const tokenParams = window.location.href.split("#")[1].split("&")

      let accessToken = tokenParams.filter((param) => param.indexOf('access_token') != -1)[0]

      if (accessToken != null) {
        accessToken = accessToken.split("=")[1]

        localStorage.setItem("ds_access_token", accessToken)

        window.close()
      }
    }

  }
}