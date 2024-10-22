import { InitOutput, WasmEmulator } from "../../pkg/ds_emulator_wasm"

const FPS_INTERVAL = 1000 / 60

const SCREEN_WIDTH = 256
const SCREEN_HEIGHT = 192

export class Renderer {
  emulator: WasmEmulator
  frames = 0
  previousTime = 0
  realPreviousTime = 0

  topCanvas: HTMLCanvasElement
  bottomCanvas: HTMLCanvasElement

  topContext: CanvasRenderingContext2D
  bottomContext: CanvasRenderingContext2D
  mouseDown = false

  animationFrameId = 0

  wasm: InitOutput
  constructor(
    emulator: WasmEmulator,
    topCanvas: HTMLCanvasElement,
    bottomCanvas: HTMLCanvasElement,
    topContext: CanvasRenderingContext2D,
    bottomContext: CanvasRenderingContext2D,
    wasm: InitOutput
  ) {
    this.emulator = emulator

    this.topCanvas = topCanvas
    this.bottomCanvas = bottomCanvas

    this.bottomContext = bottomContext
    this.topContext = topContext

    this.wasm = wasm
  }

  run(time: number, callback: () => void, errorCallback: () => void) {
    const diff = time - this.previousTime
    const realDiff = time - this.realPreviousTime

    if (this.frames == 60) {
      this.frames = 0
      const fpsCounter = document.getElementById("fps-counter")

      if (fpsCounter != null) {
        fpsCounter.innerHTML = `FPS = ${1000 / diff}`
      }
    }

    this.realPreviousTime = time

    if (diff >= FPS_INTERVAL || this.previousTime == 0) {
      try {
        this.emulator.step_frame()
      } catch (e: any) {
        this.cancelRendering()
        errorCallback()

        console.log(e)
      }

      this.frames++
      this.previousTime = time - (diff % FPS_INTERVAL)

      callback()

      let topPointer = null
      let bottomPointer = null

      if (this.emulator.is_top_a()) {
        topPointer = this.emulator.get_engine_a_picture_pointer()
        bottomPointer = this.emulator.get_engine_b_picture_pointer()
      } else {
        topPointer = this.emulator.get_engine_b_picture_pointer()
        bottomPointer = this.emulator.get_engine_a_picture_pointer()
      }
      this.updatePicture(topPointer, this.topContext)
      this.updatePicture(bottomPointer, this.bottomContext)
    }

    this.animationFrameId = requestAnimationFrame((time) => this.run(time, callback, errorCallback))
  }

  cancelRendering() {
    cancelAnimationFrame(this.animationFrameId)
  }

  updatePicture(pointer: number, currentContext: CanvasRenderingContext2D) {
    const engineBuffer = new Uint8Array(this.wasm.memory.buffer, pointer)

    const imageData = currentContext.getImageData(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)

    for (let y = 0; y < SCREEN_HEIGHT; y++) {
      for (let x = 0; x < SCREEN_WIDTH; x++) {
        const imageIndex = x * 4 + y * 4 * SCREEN_WIDTH

        imageData.data[imageIndex] = engineBuffer[imageIndex]
        imageData.data[imageIndex+1] = engineBuffer[imageIndex+1]
        imageData.data[imageIndex+2] = engineBuffer[imageIndex+2]
        imageData.data[imageIndex+3] = engineBuffer[imageIndex+3]
      }
    }
    currentContext.putImageData(imageData, 0, 0)
  }

  setCursorPosition(event: MouseEvent) {
    const rect = this.bottomCanvas.getBoundingClientRect()
    const x = event.clientX - rect.left
    const y = event.clientY - rect.top

    const widthRatio = SCREEN_WIDTH / rect.width
    const heightRatio = SCREEN_HEIGHT / rect.height

    this.emulator?.touch_screen(x * widthRatio, y * heightRatio)
  }

  addCanvasListeners() {
    this.bottomCanvas.addEventListener('mousedown', (e) => {
      this.mouseDown = true
      this.setCursorPosition(e)
    })

    this.bottomCanvas.addEventListener('mouseup', (e) => {
      this.emulator?.release_screen()
      this.mouseDown = false
    })

    this.bottomCanvas.addEventListener('mousemove', (e) => {
      if (this.mouseDown) {
        this.setCursorPosition(e)
      }
    })

    this.bottomCanvas.addEventListener('mouseout', (e) => {
      if (this.mouseDown) {
        this.emulator?.release_screen()
        this.mouseDown = false
      }
    })
  }
}