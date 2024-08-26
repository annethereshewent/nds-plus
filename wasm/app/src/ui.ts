import init, { WasmEmulator, InitOutput } from "../../pkg/ds_emulator_wasm.js"
import JSZip from 'jszip'
import { DsDatabase } from "./ds_database"
import { Audio } from "./audio"
import { Renderer } from "./renderer"
import { Joypad } from "./joypad"
import wasmData from '../../pkg/ds_emulator_wasm_bg.wasm'

interface GameDBEntry {
  save_type: string,
  ram_capacity: number,
  game_code: number
}

export class UI {
  emulator: WasmEmulator|null = null

  biosData7: Uint8Array|null = null
  biosData9: Uint8Array|null = null
  firmware: Uint8Array|null = null

  fileName = ""
  gameData: Uint8Array|null = null

  updateSaveGame = ""

  db: DsDatabase = new DsDatabase()

  wasm: InitOutput|null = null

  audio: Audio|null = null
  renderer: Renderer|null = null

  timeout: any|null = null

  keyboardButtons: boolean[] = []

  joypad: Joypad|null = null

  constructor() {
    const bios7Json = JSON.parse(localStorage.getItem("ds_bios7") || "null")
    const bios9Json = JSON.parse(localStorage.getItem("ds_bios9") || "null")
    const firmwareJson = JSON.parse(localStorage.getItem("ds_firmware") || "null")

    if (bios7Json != null) {
      this.biosData7 = new Uint8Array(bios7Json)
    }

    if (bios9Json != null) {
      this.biosData9 = new Uint8Array(bios9Json)
    }

    if (firmwareJson != null) {
      this.firmware = new Uint8Array(firmwareJson)
    }

    if (this.biosData7 != null && this.biosData9 != null && this.firmware != null) {
      document.getElementById("game-button")?.removeAttribute("disabled")
      document.getElementById("bios9-button")?.setAttribute("disabled", "true")
      document.getElementById("bios7-button")?.setAttribute("disabled", "true")
      document.getElementById("firmware-button")?.setAttribute("disabled", "true")
    }
  }

  async setWasm() {
    this.wasm = await init(wasmData)
  }

  addEventListeners() {
    document.getElementById("bios7-button")?.addEventListener("click", () => document.getElementById("bios7-input")?.click())
    document.getElementById("bios9-button")?.addEventListener("click", () => document.getElementById("bios9-input")?.click())
    document.getElementById("firmware-button")?.addEventListener("click", () => document.getElementById("firmware-input")?.click())
    document.getElementById("game-button")?.addEventListener("click", () => document.getElementById("game-input")?.click())

    document.getElementById("bios7-input")?.addEventListener("change", (e) => this.handleBios7Change(e))
    document.getElementById("bios9-input")?.addEventListener("change", (e) => this.handleBios9Change(e))
    document.getElementById("firmware-input")?.addEventListener("change", (e) => this.handleFirmwareChange(e))
    document.getElementById("game-input")?.addEventListener("change", (e) => this.handleGameChange(e))
    document.getElementById("save-input")?.addEventListener("change", (e) => this.handleSaveChange(e))

    document.getElementById("manage-saves-button")?.addEventListener("click", () => this.displaySavesModal())
  }

  async displaySavesModal() {
    const saves = await this.db.getSaves()
    const savesModal = document.getElementById("saves-modal")
    const savesList = document.getElementById("saves-list")

    if (saves != null && savesModal != null && savesList != null) {
      savesModal.className = "modal show"
      savesModal.style.display = "block"

      savesList.innerHTML = ''
      for (const save of saves) {
        const divEl = document.createElement("div")

        divEl.className = "save-entry"

        const spanEl = document.createElement("span")

        spanEl.innerText = save.gameName

        const deleteSaveEl = document.createElement('i')

        deleteSaveEl.className = "fa-solid fa-x save-icon delete-save"

        deleteSaveEl.addEventListener('click', () => this.deleteSave(save.gameName))

        const updateSaveEl = document.createElement('i')

        updateSaveEl.className = "fa-solid fa-file-pen save-icon update"

        updateSaveEl.addEventListener("click", () => this.updateSave(save.gameName))

        const downloadSaveEl = document.createElement("div")

        downloadSaveEl.className = "fa-solid fa-download save-icon download"

        downloadSaveEl.addEventListener("click", () => this.downloadSave(save.gameName))

        divEl.append(spanEl)
        divEl.append(downloadSaveEl)
        divEl.append(deleteSaveEl)
        divEl.append(updateSaveEl)

        savesList.append(divEl)
      }
    }
  }

  updateSave(gameName: string) {
    this.updateSaveGame = gameName

    document.getElementById("save-input")?.click()
  }

  async downloadSave(gameName: string) {
    const entry = await this.db.getSave(gameName)

    if (entry != null) {
      this.generateFile(entry.data, gameName)
    }
  }

  generateFile(data: Uint8Array, gameName: string) {
    const blob = new Blob([data], {
      type: "application/octet-stream"
    })

    const objectUrl = URL.createObjectURL(blob)

    const a = document.createElement('a')

    a.href = objectUrl
    a.download = `${gameName}.sav`
    document.body.append(a)
    a.style.display = "none"

    a.click()
    a.remove()

    setTimeout(() => URL.revokeObjectURL(objectUrl), 1000)
  }

  async deleteSave(gameName: string) {
    if (confirm("are you sure you want to delete this save?")) {
      const result = await this.db.deleteSave(gameName)

      if (result) {
        const savesList = document.getElementById("saves-list")

        if (savesList != null) {
          for (const child of savesList.children) {
            const children = [...child.children]
            const spanElement = (children.filter((childEl) => childEl.tagName.toLowerCase() == 'span')[0] as HTMLSpanElement)

            if (spanElement?.innerText == gameName) {
              child.remove()
              break
            }
          }
        }
      }
    }
  }

  async handleSaveChange(e: Event) {
    const data = await this.getBinaryData(e)

    if (data != null) {
      const bytes = new Uint8Array(data as ArrayBuffer)

      if (this.updateSaveGame != "") {
        this.db.setSave(this.updateSaveGame, bytes)
      }

    }
  }

  async saveGame() {
    if (this.wasm != null) {
      let saveData = new Uint8Array(this.wasm.memory.buffer, this.emulator!!.backup_pointer(), this.emulator!!.backup_length())

      let gameName = this.fileName.split('/').pop()

      if (gameName != null) {
        gameName = gameName.substring(0, gameName.lastIndexOf('.'))

        this.db.setSave(gameName, saveData)
      }
    }
  }

  async handleGameChange(e: Event) {
    const game = await this.getBinaryData(e, true)

    if (game != null) {
      this.gameData = new Uint8Array(game as ArrayBuffer)

      const response = await fetch("./game_db.json")

      const gameDbJson = await response.json()

      this.emulator = new WasmEmulator(this.biosData7!!, this.biosData9!!, this.firmware!!, this.gameData)
      this.joypad = new Joypad(this.emulator)
      this.joypad.addKeyboardEventListeners()

      const gameCode = this.emulator.get_game_code()

      const entry = gameDbJson.filter((entry: GameDBEntry) => entry.game_code == gameCode)[0]

      if (entry != null) {
        let bytes = new Uint8Array(0)

        let gameName = this.fileName.split('/').pop()
        gameName = gameName?.substring(0, gameName.lastIndexOf('.'))

        if (gameName != null) {
          const saveEntry = await this.db.getSave(gameName)
          const saveJson = JSON.parse(localStorage.getItem(gameName) || "null")

          if (saveEntry != null) {
            this.emulator.set_backup(entry.save_type, entry.ram_capacity, saveEntry.data)
          } else if (saveJson != null) {
            bytes = new Uint8Array(saveJson)

            if (gameName != null) {
              await this.db.setSave(gameName, bytes)
            }

            localStorage.removeItem(gameName)

            this.emulator.set_backup(entry.save_type, entry.ram_capacity, bytes)
          } else {
            this.emulator.set_backup(entry.save_type, entry.ram_capacity, bytes)
          }
        }

      } else {
        alert("Couldn't find game in DB, resorting to no save")
      }

      const topCanvas = document.getElementById("top-canvas") as HTMLCanvasElement
      const bottomCanvas = document.getElementById("bottom-canvas") as HTMLCanvasElement

      const topContext = topCanvas.getContext("2d")
      const bottomContext = bottomCanvas.getContext("2d")

      this.audio = new Audio(this.emulator)

      if (topContext != null && bottomContext != null && this.wasm != null) {
        this.renderer = new Renderer(
          this.emulator,
          topCanvas,
          bottomCanvas,
          topContext,
          bottomContext,
          this.wasm
        )

        this.renderer.addCanvasListeners()
      } else {
        throw new Error("could not initialize canvases for rendering")
      }

      this.audio.startAudio()

      requestAnimationFrame((time) => this.renderer?.run(time, () => {
        this.joypad?.handleJoypadInput()
        this.checkSaves()
      }))
    }
  }

  checkSaves() {
    if (this.emulator?.has_saved()) {
      this.emulator?.set_saved(false)
      clearTimeout(this.timeout)
      this.timeout = setTimeout(this.saveGame, 1000)
    }
  }

  async handleFirmwareChange(e: Event) {
    this.firmware = await this.handleFileChange(e)

    localStorage.setItem("ds_firmware", JSON.stringify(Array.from(this.firmware || [])))

    this.checkIfAllLoaded()
  }

  async handleBios9Change(e: Event) {
    this.biosData9 = await this.handleFileChange(e)

    localStorage.setItem("ds_bios9", JSON.stringify(Array.from(this.biosData9 || [])))

    this.checkIfAllLoaded()
  }

  checkIfAllLoaded() {
    if (this.biosData7 != null && this.biosData9 != null && this.firmware != null) {
      document.getElementById("game-button")?.removeAttribute("disabled")
      document.getElementById("bios9-button")?.setAttribute("disabled", "true")
      document.getElementById("bios7-button")?.setAttribute("disabled", "true")
      document.getElementById("firmware-button")?.setAttribute("disabled", "true")
    }
  }

  async handleBios7Change(e: Event) {
    this.biosData7 = await this.handleFileChange(e)

    localStorage.setItem("ds_bios7", JSON.stringify(Array.from(this.biosData7 || [])))

    this.checkIfAllLoaded()
  }

  async handleFileChange(e: Event) {
    let fileData = null
    const data = await this.getBinaryData(e)

    if (data != null) {
      fileData = new Uint8Array(data as ArrayBuffer)

      const fileNotification = document.getElementById("bios-notification")

      if (fileNotification != null) {
        fileNotification.style.display = "block"

        let opacity = 1.0

        let interval = setInterval(() => {
          opacity -= 0.1

          fileNotification.style.opacity = `${opacity}`

          if (opacity <= 0) {
            clearInterval(interval)
          }
        }, 100)
      }

    }
    return fileData
  }

  fileToArrayBuffer(file: File): Promise<string|ArrayBuffer|null|undefined> {
    const fileReader = new FileReader()

    return new Promise((resolve, reject) => {
      fileReader.onload = () => resolve(fileReader.result)

      fileReader.onerror = () => {
        fileReader.abort()
        reject(new Error("Error parsing file"))
      }

      fileReader.readAsArrayBuffer(file)
    })
  }

  async getBinaryData(e: Event, setFilename: boolean = false): Promise<string|ArrayBuffer|null|undefined> {
    let data = null
    if ((e.target as HTMLInputElement)?.files != null) {
      const files = (e.target as HTMLInputElement)?.files

      if (files != null) {
        if (setFilename) {
          this.fileName = files[0].name
        }
        if (files[0].name.indexOf(".zip") !== -1) {
          // unzip the file first
          const zipFile = await JSZip.loadAsync(files[0])
          const zipFileName = Object.keys(zipFile.files)[0]

          data = await zipFile?.file(zipFileName)?.async('arraybuffer')
        } else {
          data = await this.fileToArrayBuffer(files[0])
        }
      }
    }

    return data
  }
}