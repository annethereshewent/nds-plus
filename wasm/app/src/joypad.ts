import { WasmEmulator } from "../../pkg/ds_emulator_wasm"
import { ButtonEvent } from "../../pkg/ds_emulator_wasm"
import { StateManager } from "./state_manager"
import { UI } from "./ui"

const CROSS_BUTTON = 0
const CIRCLE_BUTTON = 1
const SQUARE_BUTTON = 2
const TRIANGLE_BUTTON = 3

const L1_BUTTON = 4
const R1_BUTTON = 5

const SELECT = 8
const START = 9

const UP = 12
const DOWN = 13
const LEFT = 14
const RIGHT = 15

const L2_BUTTON = 6
const R2_BUTTON = 7

const L3_BUTTON = 10
const R3_BUTTON = 11

const PS_BUTTON = 16

export class Joypad {
  private emulator: WasmEmulator
  private keyboardButtons: boolean[] = []

  private useControlStick = false

  private updatingControlStick = false
  private updatingSaveState = false

  private stateManager: StateManager
  private ui: UI

  constructor(emulator: WasmEmulator, ui: UI) {
    this.emulator = emulator
    this.stateManager = ui.stateManager!
    this.ui = ui
  }

  handleJoypadInput() {
    const gamepad = navigator.getGamepads()[0]

    this.emulator?.update_input(ButtonEvent.Select, gamepad?.buttons[SELECT].pressed == true || this.keyboardButtons[SELECT])
    this.emulator?.update_input(ButtonEvent.Start, gamepad?.buttons[START].pressed == true || this.keyboardButtons[START])
    this.emulator?.update_input(ButtonEvent.Up, gamepad?.buttons[UP].pressed == true || this.keyboardButtons[UP])
    this.emulator?.update_input(ButtonEvent.Right, gamepad?.buttons[RIGHT].pressed == true || this.keyboardButtons[RIGHT])
    this.emulator?.update_input(ButtonEvent.Down, gamepad?.buttons[DOWN].pressed == true || this.keyboardButtons[DOWN])
    this.emulator?.update_input(ButtonEvent.Left, gamepad?.buttons[LEFT].pressed == true || this.keyboardButtons[LEFT])
    this.emulator?.update_input(ButtonEvent.ButtonL, gamepad?.buttons[L1_BUTTON].pressed == true || this.keyboardButtons[L1_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonR, gamepad?.buttons[R1_BUTTON].pressed == true || this.keyboardButtons[R1_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonX, gamepad?.buttons[TRIANGLE_BUTTON].pressed == true || this.keyboardButtons[TRIANGLE_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonA, gamepad?.buttons[CIRCLE_BUTTON].pressed == true || this.keyboardButtons[CIRCLE_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonB, gamepad?.buttons[CROSS_BUTTON].pressed == true || this.keyboardButtons[CROSS_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonY, gamepad?.buttons[SQUARE_BUTTON].pressed == true || this.keyboardButtons[SQUARE_BUTTON])


    if (this.useControlStick) {
      this.emulator?.touch_screen_controller(gamepad?.axes[0] || 0.0, gamepad?.axes[1] || 0.0)
    }

    if (gamepad?.buttons[R3_BUTTON].pressed && !this.updatingControlStick){
      this.updatingControlStick = true
      this.useControlStick = !this.useControlStick

      this.updateAnalogStatus()

      if (this.useControlStick) {
        this.emulator.press_screen()
      } else {
        this.emulator.release_screen()
      }
      setTimeout(() => {
        this.updatingControlStick = false
      }, 300)
    }

    if (gamepad?.buttons[L2_BUTTON].pressed && !this.updatingSaveState) {
      this.updatingSaveState = true

      this.createSaveState()

      setTimeout(() => {
        this.updatingSaveState = false
      }, 300)
    }

    if (gamepad?.buttons[R2_BUTTON].pressed && !this.updatingSaveState) {
      this.updatingSaveState = true

      this.loadSaveState()

      setTimeout(() => {
        this.updatingSaveState = false
      }, 300)
    }
  }

  updateAnalogStatus() {
    const element = document.getElementById("analog-mode")

    if (element != null) {
      for (const child of element.children) {
        if (child.tagName.toLowerCase() == "span") {
          const status = this.useControlStick ? "On" : "Off"
          child.innerHTML = `<label>Analog Mode</label>: ${status}`
        }
        if (child.id == "analog-mode-status") {
          (child as HTMLElement).style.background = this.useControlStick ? "#50C878" : "#D70040"
        }
      }

    }
  }

  createSaveState() {
    const imageUrl = this.ui.getImageUrl()
    if (imageUrl != null) {
      this.stateManager.createSaveState(imageUrl)
      .then(() => {
        this.ui.showStateCreatedNotification()
      })
    }
  }

  async loadSaveState() {
    const data = await this.stateManager.getSaveStateData()

    if (data != null) {
      this.ui.loadSaveState(data)
    }
  }

  addKeyboardEventListeners() {
    document.addEventListener("keydown", (e) => {
      switch (e.key) {
        case "w":
          this.keyboardButtons[UP] = true
          break
        case "a":
          this.keyboardButtons[LEFT] = true
          break
        case "s":
          this.keyboardButtons[DOWN] = true
          break
        case "d":
          this.keyboardButtons[RIGHT] = true
          break
        case "k":
          this.keyboardButtons[CROSS_BUTTON] = true
          break
        case "l":
          this.keyboardButtons[CIRCLE_BUTTON] = true
          break
        case "j":
          this.keyboardButtons[SQUARE_BUTTON] = true
          break
        case "i":
          this.keyboardButtons[TRIANGLE_BUTTON] = true
          break
        case "c":
          this.keyboardButtons[L1_BUTTON] = true
          break
        case "v":
          this.keyboardButtons[R1_BUTTON] = true
          break
        case "t":
          this.useControlStick = !this.useControlStick

          if (this.useControlStick) {
            this.emulator.press_screen()
          } else {
            this.emulator.release_screen()
          }
          break
        case "Enter":
          e.preventDefault()
          this.keyboardButtons[START] = true
          break
        case "Tab":
          e.preventDefault()
          this.keyboardButtons[SELECT] = true
          break
        case "F5":
          e.preventDefault()
          this.createSaveState()
          break
        case "F7":
          e.preventDefault()
          this.loadSaveState()
          break
      }
    })

    document.addEventListener("keyup", (e) => {
      switch (e.key) {
        case "w":
          this.keyboardButtons[UP] = false
          break
        case "a":
          this.keyboardButtons[LEFT] = false
          break
        case "s":
          this.keyboardButtons[DOWN] = false
          break
        case "d":
          this.keyboardButtons[RIGHT] = false
          break
        case "k":
          this.keyboardButtons[CROSS_BUTTON] = false
          break
        case "l":
          this.keyboardButtons[CIRCLE_BUTTON] = false
          break
        case "j":
          this.keyboardButtons[SQUARE_BUTTON] = false
          break
        case "i":
          this.keyboardButtons[TRIANGLE_BUTTON] = false
          break
        case "c":
          this.keyboardButtons[L1_BUTTON] = false
          break
        case "v":
          this.keyboardButtons[R1_BUTTON] = false
          break
        case "Enter":
          e.preventDefault()
          this.keyboardButtons[START] = false
          break
        case "Tab":
          e.preventDefault()
          this.keyboardButtons[SELECT] = false
          break
      }
    })
  }
}