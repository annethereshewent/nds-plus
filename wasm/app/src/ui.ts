import init, { WasmEmulator, InitOutput } from "../../pkg/ds_emulator_wasm.js"
import JSZip from 'jszip'
import { DsDatabase } from "./ds_database"
import { Audio } from "./audio"
import { Renderer, SCREEN_HEIGHT, SCREEN_WIDTH } from "./renderer"
import { Joypad } from "./joypad"
import wasmData from '../../pkg/ds_emulator_wasm_bg.wasm'
import { CloudService } from "./cloud_service"
import { StateManager } from "./state_manager"
import moment from 'moment'
import { StateEntry } from "./game_state_entry.js"

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
  gameName = ""
  gameData: Uint8Array|null = null

  updateSaveGame = ""

  db: DsDatabase

  wasm: InitOutput|null = null

  audio: Audio|null = null
  renderer: Renderer|null = null

  timeout: any|null = null

  keyboardButtons: boolean[] = []

  joypad: Joypad|null = null

  cloudService = new CloudService()

  stateManager: StateManager|null = null

  constructor() {
    const bios7Json = JSON.parse(localStorage.getItem("ds_bios7") || "null")
    const bios9Json = JSON.parse(localStorage.getItem("ds_bios9") || "null")
    const firmwareJson = JSON.parse(localStorage.getItem("ds_firmware") || "null")

    if (bios7Json != null) {
      this.biosData7 = new Uint8Array(bios7Json)
      document.getElementById("bios7-button")?.setAttribute("disabled", "true")
    }

    if (bios9Json != null) {
      this.biosData9 = new Uint8Array(bios9Json)
      document.getElementById("bios9-button")?.setAttribute("disabled", "true")
    }

    if (firmwareJson != null) {
      this.firmware = new Uint8Array(firmwareJson)
      document.getElementById("firmware-button")?.setAttribute("disabled", "true")
    }

    if (this.biosData7 != null && this.biosData9 != null && this.firmware != null) {
      document.getElementById("game-button")?.removeAttribute("disabled")
    }

    this.db = new DsDatabase()
  }

  async setWasm() {
    this.wasm = await init(wasmData)
  }

  checkOauth() {
    this.cloudService.checkAuthentication()
  }

  async uploadLocalSaves() {
    if (this.cloudService.usingCloud &&
      confirm("This will overwrite any saves on the cloud. Are you sure you want to continue?")
    ) {
      // check for any local saves and upload them.
      const saveEntries = await this.db.getSaves()

      if (saveEntries != null) {
        for (const entry of saveEntries) {
          this.cloudService.uploadSave(entry.gameName, entry.data!!)
          // delete from indexeddb as they're already stored in the cloud.
          // only time it will fall back to indexeddb is if user
          // is not using cloud
          this.db.deleteSave(entry.gameName)
        }
      }
    }
  }

  closeStatesModal() {
    this.emulator?.set_pause(false)
    const statesModal = document.getElementById("states-modal")

    if (statesModal != null) {
      statesModal.className = "modal hide"
      statesModal.style.display = "none"
    }
  }

  closeSavesModal() {
    this.emulator?.set_pause(false)
    const savesModal = document.getElementById("saves-modal")

    if (savesModal != null) {
      savesModal.className = "modal hide"
      savesModal.style.display = "none"
    }
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

    document.getElementById("save-management")?.addEventListener("click", () => this.displaySavesModal())
    document.getElementById("save-states")?.addEventListener("click", () => this.displaySaveStatesModal())
    document.getElementById("create-save-state")?.addEventListener("click", () => this.createSaveState())
    document.getElementById("states-modal-close")?.addEventListener("click", () => this.closeStatesModal())
    document.getElementById("hide-saves-modal")?.addEventListener("click", () => this.closeSavesModal())

    document.addEventListener("keydown", (e) => {
      if (e.key == 'Escape') {
        e.preventDefault()

        this.emulator?.set_pause(false)
        const helpModal = document.getElementById("help-modal")
        if (helpModal != null) {
          helpModal.className = "modal hide"
        }

        const savesModal = document.getElementById("saves-modal")

        if (savesModal != null) {
          savesModal.className = "modal hide"
          savesModal.style.display = "none"
        }

        const statesModal = document.getElementById("states-modal")

        if (statesModal != null) {
          statesModal.className = "modal hide"
          statesModal.style.display = "none"
        }
      }
    })
  }

  async createSaveState() {
    const now = moment()

    const stateName = `${now.unix()}.state`

    if (this.gameName != "") {
      const imageBytes = this.getImageBytes()
      const entry = await this.stateManager?.createSaveState(imageBytes, stateName)
      const statesList = document.getElementById("states-list")

      if (entry != null && statesList != null) {
        this.addStateElement(statesList, entry.states[stateName])
      }
    }
  }

  displayMenu(stateName: string) {
    const menu = document.getElementById(`menu-${stateName}`)

    if (menu != null) {
      if (menu.style.display == "block") {
        menu.style.display = "none"
      } else {
        menu.style.display = "block"
      }
    }
  }

  addStateElement(statesList: HTMLElement, entry: StateEntry) {
    const divEl = document.createElement("div")

    divEl.className = "state-element"
    divEl.id = entry.stateName

    divEl.addEventListener("click", () => this.displayMenu(entry.stateName))

    const imgEl = document.createElement("img")

    imgEl.className = "state-image"

    const imgBlob = new Blob([entry.imageBytes])

    const pEl = document.createElement("p")

    if (entry.stateName != "quick_save.state") {

      const timestamp = parseInt(entry.stateName.replace(".state", ""))

      pEl.innerText = `Save on ${moment.unix(timestamp).format("lll")}`
    } else {
      pEl.innerText = "Quick save"
    }

    const menu = document.createElement("aside")

    menu.className = "state-menu hide"
    menu.id = `menu-${entry.stateName}`
    menu.style.display = "none"

    menu.innerHTML = `
      <ul class="state-menu-list">
        <li><a id="load-${entry.stateName}">Load state</a></li>
        <li><a id="delete-${entry.stateName}">Delete state</a></li>
      </ul>
    `
    imgEl.src = URL.createObjectURL(imgBlob)


    divEl.append(imgEl)
    divEl.append(menu)
    divEl.append(pEl)

    statesList.append(divEl)

    // finally add event listeners for loading and deleting states
    document.getElementById(`load-${entry.stateName}`)?.addEventListener("click", () => this.loadState(entry.state))
    document.getElementById(`delete-${entry.stateName}`)?.addEventListener("click", () => this.deleteState(entry.stateName))
  }

  async displaySavesModal() {
    const saves = !this.cloudService.usingCloud ? await this.db.getSaves() : await this.cloudService.getSaves()
    const savesModal = document.getElementById("saves-modal")
    const savesList = document.getElementById("saves-list")

    if (saves != null && savesModal != null && savesList != null) {
      savesModal.className = "modal show"
      savesModal.style.display = "block"

      this.emulator?.set_pause(true)

      savesList.innerHTML = ''
      for (const save of saves) {
        const divEl = document.createElement("div")

        divEl.className = "save-entry"

        const spanEl = document.createElement("span")

        spanEl.innerText = save.gameName.length > 50 ? save.gameName.substring(0, 50) + "..." : save.gameName

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

      const hasSaves = await this.db.hasSaves()
      if (this.cloudService.usingCloud && hasSaves) {
        const localSavesEl = document.createElement("div")

        const button = document.createElement("button")

        button.innerText = "Upload local saves"

        button.className = "button is-danger is-small"

        button.addEventListener("click", () => this.uploadLocalSaves())

        localSavesEl.style.textAlign = "center"
        localSavesEl.style.marginTop = "20px"
        localSavesEl.append(button)

        savesList.append(localSavesEl)
      }
    }
  }

  async displaySaveStatesModal() {
    if (this.gameName != "") {
      const modal = document.getElementById("states-modal")
      const statesList = document.getElementById("states-list")

      if (modal != null && statesList != null) {
        this.emulator?.set_pause(true)
        modal.style.display = "block"

        statesList.innerHTML = ""

        const entry = await this.db.getSaveStates(this.gameName)

        if (entry != null) {
          for (const key in entry.states) {
            const stateEntry = entry.states[key]

            this.addStateElement(statesList, stateEntry)
          }
        }
      }
    }
  }

  updateSave(gameName: string) {
    this.updateSaveGame = gameName

    document.getElementById("save-input")?.click()
  }

  async downloadSave(gameName: string) {
    const entry = !this.cloudService.usingCloud ?  await this.db.getSave(gameName) : await this.cloudService.getSave(gameName)

    if (entry != null) {
      this.generateFile(entry.data!!, gameName)
    }
  }

  getImageBytes() {
    if (this.emulator != null && this.wasm != null) {
      let imageBytes = new Uint8Array()
      if (this.emulator.is_top_a()) {
        imageBytes = new Uint8Array(this.wasm.memory.buffer, this.emulator.get_engine_a_picture_pointer(), SCREEN_WIDTH * SCREEN_HEIGHT * 4)

        const lowerScreen = new Uint8Array(this.wasm.memory.buffer, this.emulator.get_engine_b_picture_pointer(), SCREEN_WIDTH * SCREEN_HEIGHT * 4)

        imageBytes = new Uint8Array([...imageBytes, ...lowerScreen])
      } else {
        imageBytes = new Uint8Array(this.wasm.memory.buffer, this.emulator.get_engine_b_picture_pointer(), SCREEN_WIDTH * SCREEN_HEIGHT * 4)

        const lowerScreen = new Uint8Array(this.wasm.memory.buffer, this.emulator.get_engine_a_picture_pointer(), SCREEN_WIDTH * SCREEN_HEIGHT * 4)

        imageBytes = new Uint8Array([...imageBytes, ...lowerScreen])
      }


      const headerSize = 70
      // finally add header and dimension information
      let headerArray = new Uint8Array(headerSize)
      const view = new DataView(headerArray.buffer)

      // see https://stackoverflow.com/questions/50620821/uint8array-to-image-in-javascript


      // magic value
      view.setUint16(0, 0x424D, false)
      // File size.
      view.setUint32(2, imageBytes.length, true)
      // Offset to image data.
      view.setUint32(10, headerSize, true)
      // Size of BITMAPINFOHEADER
      view.setUint32(14, 40, true);
      // Width
      view.setInt32(18, SCREEN_WIDTH, true)
      // Height (signed because negative values flip
      // the image vertically).
      view.setInt32(22, -(SCREEN_HEIGHT * 2), true)
      // Number of color planes (colors stored as
      // separate images; must be 1).
      view.setUint16(26, 1, true)
      // Bits per pixel.
      view.setUint16(28, 32, true)
      // Compression method, 6 = BI_ALPHABITFIELDS
      view.setUint32(30, 6, true)
      // Image size in bytes.
      view.setUint32(34, SCREEN_WIDTH * (SCREEN_HEIGHT * 2) * 4, true)
      // Horizontal resolution, pixels per metre.
      // This will be unused in this situation.
      view.setInt32(38, 10000, true)
      // Vertical resolution, pixels per metre.
      view.setInt32(42, 10000, true)
      // Number of colors. 0 = all
      view.setUint32(46, 0, true)
      // Number of important colors. 0 = all
      view.setUint32(50, 0, true)

      // color table. Because we used BI_ALPHABITFIELDS
      // this specifies the R, G, B and A bitmasks.

      // Red
      view.setUint32(54, 0x000000FF, true)
      // Green
      view.setUint32(58, 0x0000FF00, true)
      // Blue
      view.setUint32(62, 0x00FF0000, true)
      // Alpha
      view.setUint32(66, 0xFF000000, true)

      headerArray = new Uint8Array(view.buffer)

      imageBytes = new Uint8Array([...headerArray, ...imageBytes])

      return imageBytes
    }

    return new Uint8Array()
  }

  generateFile(data: Uint8Array, gameName: string) {
    const blob = new Blob([data], {
      type: "application/octet-stream"
    })

    const objectUrl = URL.createObjectURL(blob)

    const a = document.createElement('a')

    a.href = objectUrl
    a.download = gameName.match(/\.sav$/) ? gameName : `${gameName}.sav`
    document.body.append(a)
    a.style.display = "none"

    a.click()
    a.remove()

    setTimeout(() => URL.revokeObjectURL(objectUrl), 1000)
  }

  async loadState(compressed: Uint8Array) {
    if (this.biosData7 != null && this.biosData9 != null && this.gameData != null) {
      this.emulator?.set_pause(true)
      if (this.emulator != null && this.stateManager != null) {
        const data = await this.stateManager.decompress(compressed)
        this.emulator.load_save_state(data)

        const { biosData7, biosData9, gameData } = this

        this.emulator.reload_bios(biosData7, biosData9)

        if (this.firmware != null) {
          this.emulator.reload_firmware(this.firmware)
        } else {
          this.emulator.hle_firmware()
        }

        this.emulator.reload_rom(gameData)

        this.closeStatesModal()
      }
    }
  }

  async deleteState(stateName: string) {
    if (confirm("Are you sure you want to delete this save state?")) {
      await this.db.deleteState(this.gameName, stateName)

      const el = document.getElementById(stateName)

      el?.remove()
    }
  }

  async deleteSave(gameName: string) {
    if (confirm("are you sure you want to delete this save?")) {
      const result = !this.cloudService.usingCloud ? await this.db.deleteSave(gameName) : await this.cloudService.deleteSave(gameName)

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
    let saveName = (e.target as HTMLInputElement)?.files?.[0].name?.split('/')?.pop()

    if (saveName != this.updateSaveGame) {
      if (!confirm("Warning! Save file does not match selected game name. are you sure you want to continue?")) {
        return
      }
    }
    const data = await this.getBinaryData(e)

    if (data != null) {
      const bytes = new Uint8Array(data as ArrayBuffer)

      if (this.updateSaveGame != "") {
        if (!this.cloudService.usingCloud) {
          this.db.setSave(this.updateSaveGame, bytes)
        } else {
          this.cloudService.uploadSave(this.updateSaveGame, bytes)
        }
      }

      const notification = document.getElementById("save-notification")

      if (notification != null) {
        notification.style.display = "block"

        let opacity = 1.0

        let interval = setInterval(() => {
          opacity -= 0.1
          notification.style.opacity = `${opacity}`

          if (opacity <= 0) {
            clearInterval(interval)
          }
        }, 100)
      }

      const savesModal = document.getElementById("saves-modal")

      if (savesModal != null) {
        savesModal.style.display = "none"
        savesModal.className = "modal hide"
      }
    }
  }

  async saveGame() {
    if (this.wasm != null) {
      let saveData = new Uint8Array(this.wasm.memory.buffer, this.emulator!!.backup_pointer(), this.emulator!!.backup_length())

      let gameName = this.fileName.split('/').pop()

      if (gameName != null) {
        gameName = gameName.substring(0, gameName.lastIndexOf('.'))


        if (!this.cloudService.usingCloud) {
          this.db.setSave(gameName, saveData)
        } else {
          this.cloudService.uploadSave(gameName, saveData)
        }
      }
    }
  }

  async handleGameChange(e: Event) {
    const game = await this.getBinaryData(e, true)

    if (game != null) {
      this.gameData = new Uint8Array(game as ArrayBuffer)

      const response = await fetch("./game_db.json")

      const gameDbJson = await response.json()

      // cancel any ongoing events if resetting the system
      this.audio?.stopAudio()
      this.renderer?.cancelRendering()

      if (this.biosData7 != null && this.biosData9 != null && this.firmware != null) {
        this.emulator = new WasmEmulator(this.biosData7, this.biosData9, this.firmware, this.gameData)
      } else {
        // load freebios files and hle the firmware
        const bios7Response = await fetch("./freebios/drastic_bios_arm7.bin")
        const bios7Body = await bios7Response.arrayBuffer()

        this.biosData7 = new Uint8Array(bios7Body)

        const bios9Response = await fetch("/freebios/drastic_bios_arm9.bin")
        const bios9Body = await bios9Response.arrayBuffer()

        this.biosData9 = new Uint8Array(bios9Body)

        // typescript won't let me use null, so it has to be undefined
        this.emulator = new WasmEmulator(this.biosData7, this.biosData9, undefined, this.gameData)
      }

      const gameCode = this.emulator.get_game_code()

      const entry = gameDbJson.filter((entry: GameDBEntry) => entry.game_code == gameCode)[0]

      if (entry != null) {
        let bytes = new Uint8Array(0)

        let gameName = this.fileName.split('/').pop()
        gameName = gameName?.substring(0, gameName.lastIndexOf('.'))

        if (gameName != null) {
          this.gameName = gameName
          this.stateManager = new StateManager(this.emulator!, this.wasm, this.gameName, this.db)
          this.joypad = new Joypad(this.emulator, this)
          this.joypad.addKeyboardEventListeners()
          const saveEntry = !this.cloudService.usingCloud ? await this.db.getSave(gameName) : await this.cloudService.getSave(gameName)

          if (saveEntry != null && saveEntry.data != null) {
            this.emulator.set_backup(entry.save_type, entry.ram_capacity, saveEntry.data)
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
      this.audio.startMicrophone()
      requestAnimationFrame((time) => this.renderer?.run(time, () => {
          this.joypad?.handleJoypadInput()
          this.checkSaves()
          this.audio?.updateMicBuffer()

        }, () => this.resetSystem())
      )
    }
  }

  resetSystem() {
    this.audio?.stopAudio()
    this.audio = null
    this.emulator = null
  }

  checkSaves() {
    if (this.emulator?.has_saved()) {
      this.emulator?.set_saved(false)
      clearTimeout(this.timeout)
      this.timeout = setTimeout(() => this.saveGame(), 1000)
    }
  }

  async handleFirmwareChange(e: Event) {
    this.firmware = await this.handleFileChange(e, "firmware.bin")

    if (this.firmware != null) {
      this.setLoaded("ds_firmware", this.firmware, "firmware-button")
    }
  }

  async handleBios9Change(e: Event) {
    this.biosData9 = await this.handleFileChange(e, "bios9.bin")

    if (this.biosData9 != null) {
      this.setLoaded("ds_bios9", this.biosData9, "bios9-button")
    }
  }

  checkIfAllLoaded() {
    if (this.biosData7 != null && this.biosData9 != null && this.firmware != null) {
      document.getElementById("game-button")?.removeAttribute("disabled")
    }
  }

  async handleBios7Change(e: Event) {
    this.biosData7 = await this.handleFileChange(e, "bios7.bin")

    if (this.biosData7 != null) {
      this.setLoaded("ds_bios7", this.biosData7, "bios7-button")
    }
  }

  setLoaded(key: string, data: Uint8Array, button: string) {
    localStorage.setItem(key, JSON.stringify(Array.from(data || [])))
    document.getElementById(button)?.setAttribute("disabled", "true")
    this.checkIfAllLoaded()
  }

  async handleFileChange(e: Event, fileName: string) {
    const targetFilename = (e.target as HTMLInputElement)?.files?.[0].name?.split('/')?.pop()

    if (targetFilename != fileName) {
      if (!confirm(`Warning! Current filename does not match the required filename of ${fileName}. Are you sure you want to continue?`)) {
        return null
      }
    }

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

  showStateCreatedNotification() {
    const notification = document.getElementById("state-notification")

    if (notification != null) {
      notification.style.display = "block"

      let opacity = 1.0

      let interval = setInterval(() => {
        opacity -= 0.1
        notification.style.opacity = `${opacity}`

        if (opacity <= 0) {
          clearInterval(interval)
        }
      }, 100)
    }
  }
}