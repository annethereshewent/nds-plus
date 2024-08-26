import { WasmEmulator } from "../../pkg/ds_emulator_wasm"
import { ButtonEvent } from "../../pkg/ds_emulator_wasm"

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

export class Joypad {
  private gamepad: Gamepad|null
  private emulator: WasmEmulator
  private keyboardButtons: boolean[] = []


  constructor(emulator: WasmEmulator) {
    this.gamepad = navigator.getGamepads()[0]
    this.emulator = emulator
  }

  handleJoypadInput() {
    if (this.gamepad == null) {
      this.gamepad = navigator.getGamepads()[0]
    }

    this.emulator?.update_input(ButtonEvent.Select, this.gamepad?.buttons[SELECT].pressed == true || this.keyboardButtons[SELECT])
    this.emulator?.update_input(ButtonEvent.Start, this.gamepad?.buttons[START].pressed == true || this.keyboardButtons[START])
    this.emulator?.update_input(ButtonEvent.Up, this.gamepad?.buttons[UP].pressed == true || this.keyboardButtons[UP])
    this.emulator?.update_input(ButtonEvent.Right, this.gamepad?.buttons[RIGHT].pressed == true || this.keyboardButtons[RIGHT])
    this.emulator?.update_input(ButtonEvent.Down, this.gamepad?.buttons[DOWN].pressed == true || this.keyboardButtons[DOWN])
    this.emulator?.update_input(ButtonEvent.Left, this.gamepad?.buttons[LEFT].pressed == true || this.keyboardButtons[LEFT])
    this.emulator?.update_input(ButtonEvent.ButtonL, this.gamepad?.buttons[L1_BUTTON].pressed == true || this.keyboardButtons[L1_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonR, this.gamepad?.buttons[R1_BUTTON].pressed == true || this.keyboardButtons[R1_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonX, this.gamepad?.buttons[TRIANGLE_BUTTON].pressed == true || this.keyboardButtons[TRIANGLE_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonA, this.gamepad?.buttons[CIRCLE_BUTTON].pressed == true || this.keyboardButtons[CIRCLE_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonB, this.gamepad?.buttons[CROSS_BUTTON].pressed == true || this.keyboardButtons[CROSS_BUTTON])
    this.emulator?.update_input(ButtonEvent.ButtonY, this.gamepad?.buttons[SQUARE_BUTTON].pressed == true || this.keyboardButtons[SQUARE_BUTTON])
  }

  addKeyboardEventListeners() {
    document.addEventListener("keydown", (e) => {
      e.preventDefault()

      switch (e.key) {
        case "Escape":
          const helpModal = document.getElementById("help-modal")
          if (helpModal != null) {
            helpModal.className = "modal hide"
          }

          const savesModal = document.getElementById("saves-modal")

          if (savesModal != null) {
            savesModal.className = "modal hide"
            savesModal.style.display = "none"
          }

          break
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
      }
    })

    document.addEventListener("keyup", (e) => {
      e.preventDefault()

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
      }
    })
  }
}