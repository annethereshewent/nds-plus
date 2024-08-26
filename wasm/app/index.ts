import { UI } from "./src/ui"
import 'bulma/css/bulma'
import './src/styles/app'

const ui = new UI()

ui.addEventListeners()
ui.setWasm()