use std::{
  collections::{
    HashMap,
    VecDeque
  },
  sync::{
    Arc,
    Mutex
  }
};

use ds_emulator::{
  apu::Sample,
  cpu::{
    bus::Bus, registers::{
      external_key_input_register::ExternalKeyInputRegister,
      key_input_register::KeyInputRegister
    }
  },
  gpu::{
    registers::power_control_register1::PowerControlRegister1, GPU, SCREEN_HEIGHT, SCREEN_WIDTH
  }
};
use sdl2::{
  audio::{
    AudioCallback,
    AudioDevice,
    AudioSpecDesired
  },
  controller::{
    Button,
    GameController
  },
  event::Event,
  keyboard::Keycode,
  pixels::PixelFormatEnum,
  rect::Rect,
  render::Canvas,
  video::Window,
  EventPump,
  Sdl
};

struct DsAudioCallback {
  audio_samples: Arc<Mutex<VecDeque<f32>>>
}

impl AudioCallback for DsAudioCallback {
  type Channel = f32;

  fn callback(&mut self, buf: &mut [Self::Channel]) {
    let mut audio_samples = self.audio_samples.lock().unwrap();
    let len = audio_samples.len();

    let mut last_sample = Sample { left: 0.0, right: 0.0 };

    if len > 2 {
      last_sample.left = audio_samples[len - 2];
      last_sample.right = audio_samples[len - 1];
    }

    let mut is_left_sample = true;

    for b in buf.iter_mut() {
      *b = if let Some(sample) = audio_samples.pop_front() {
        sample
      } else {
        if is_left_sample {
          last_sample.left
        } else {
          last_sample.right
        }
      };
      is_left_sample = !is_left_sample;
    }
  }
}

pub struct Frontend {
  event_pump: EventPump,
  canvas: Canvas<Window>,
  _controller: Option<GameController>,
  button_map: HashMap<Button, KeyInputRegister>,
  ext_button_map: HashMap<Button, ExternalKeyInputRegister>,
  ext_key_map: HashMap<Keycode, ExternalKeyInputRegister>,
  key_map: HashMap<Keycode, KeyInputRegister>,
  device: AudioDevice<DsAudioCallback>
}

impl Frontend {
  pub fn new(sdl_context: &Sdl, audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
      .window("DS Emulator", (SCREEN_WIDTH * 2) as u32, (SCREEN_HEIGHT * 2 * 2) as u32)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let event_pump = sdl_context.event_pump().unwrap();

    let game_controller_subsystem = sdl_context.game_controller().unwrap();

    let available = game_controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e)).unwrap();

    let _controller = (0..available)
      .find_map(|id| {
        match game_controller_subsystem.open(id) {
          Ok(c) => {
            Some(c)
          }
          Err(_) => {
            None
          }
        }
      });

    let audio_subsystem = sdl_context.audio().unwrap();

    let spec = AudioSpecDesired {
      freq: Some(44100),
      channels: Some(2),
      samples: Some(4096)
    };

    let device = audio_subsystem.open_playback(
      None,
      &spec,
      |_| DsAudioCallback { audio_samples: audio_buffer }
    ).unwrap();

    device.resume();

    let mut key_map = HashMap::new();

    key_map.insert(Keycode::W, KeyInputRegister::Up);
    key_map.insert(Keycode::S, KeyInputRegister::Down);
    key_map.insert(Keycode::D, KeyInputRegister::Right);
    key_map.insert(Keycode::A, KeyInputRegister::Left);

    key_map.insert(Keycode::Space, KeyInputRegister::ButtonA);
    key_map.insert(Keycode::K, KeyInputRegister::ButtonA);

    key_map.insert(Keycode::LShift, KeyInputRegister::ButtonB);
    key_map.insert(Keycode::J, KeyInputRegister::ButtonB);

    key_map.insert(Keycode::C, KeyInputRegister::ButtonL);
    key_map.insert(Keycode::V, KeyInputRegister::ButtonR);

    key_map.insert(Keycode::Return, KeyInputRegister::Start);
    key_map.insert(Keycode::Tab, KeyInputRegister::Select);

    let mut ext_key_map = HashMap::new();

    ext_key_map.insert(Keycode::N, ExternalKeyInputRegister::BUTTON_Y);
    ext_key_map.insert(Keycode::M, ExternalKeyInputRegister::BUTTON_X);

    let mut button_map = HashMap::new();

    button_map.insert(Button::B, KeyInputRegister::ButtonA);
    button_map.insert(Button::A, KeyInputRegister::ButtonB);

    button_map.insert(Button::Start, KeyInputRegister::Start);
    button_map.insert(Button::Back, KeyInputRegister::Select);

    button_map.insert(Button::DPadUp, KeyInputRegister::Up);
    button_map.insert(Button::DPadDown, KeyInputRegister::Down);
    button_map.insert(Button::DPadLeft, KeyInputRegister::Left);
    button_map.insert(Button::DPadRight, KeyInputRegister::Right);

    button_map.insert(Button::LeftShoulder, KeyInputRegister::ButtonL);
    button_map.insert(Button::RightShoulder, KeyInputRegister::ButtonR);

    let mut ext_button_map = HashMap::new();

    ext_button_map.insert(Button::Y, ExternalKeyInputRegister::BUTTON_X);
    ext_button_map.insert(Button::X, ExternalKeyInputRegister::BUTTON_Y);

    Self {
      event_pump,
      canvas,
      _controller,
      button_map,
      ext_button_map,
      key_map,
      ext_key_map,
      device
    }
  }

  pub fn handle_touchscreen(&mut self, bus: &mut Bus) {
    let state = self.event_pump.mouse_state();

    let y = state.y();
    let x = state.x();

    if state.left() && y >= SCREEN_HEIGHT as i32 * 2 && x >= 0 {
      bus.arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
      bus.touchscreen.touch_screen(x as u16 / 2, y as u16 / 2 - SCREEN_HEIGHT);
    } else if !state.left() {
      bus.touchscreen.release_screen();
      bus.arm7.extkeyin.insert(ExternalKeyInputRegister::PEN_DOWN);
    }
  }

  pub fn handle_events(&mut self, bus: &mut Bus) {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => std::process::exit(0),
        Event::KeyDown { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            bus.key_input_register.set(*button, false);
          } else if let Some(button) = self.ext_key_map.get(&keycode.unwrap()) {
            bus.arm7.extkeyin.set(*button, false);
          } else if keycode.unwrap() == Keycode::G {
            bus.debug_on = !bus.debug_on
          } else if keycode.unwrap() == Keycode::F {
            let mut engine_a = bus.gpu.engine_a.lock().unwrap();
            let mut engine_b = bus.gpu.engine_b.lock().unwrap();
            let mut engine3d = bus.gpu.engine3d.lock().unwrap();

            engine_a.debug_on = !engine_a.debug_on;
            engine_b.debug_on = !engine_b.debug_on;
            engine3d.debug_on = !engine3d.debug_on;
          }
        }
        Event::KeyUp { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            bus.key_input_register.set(*button, true);
          } else if let Some(button) = self.ext_key_map.get(&keycode.unwrap()) {
            bus.arm7.extkeyin.set(*button, true);
          }
        }
        Event::ControllerButtonDown { button, .. } => {
          if let Some(button) = self.ext_button_map.get(&button) {
            bus.arm7.extkeyin.set(*button, false);
          } else if let Some(button) = self.button_map.get(&button) {
            bus.key_input_register.set(*button, false);
          }
        }
        Event::ControllerButtonUp { button, .. } => {
          if let Some(button) = self.ext_button_map.get(&button) {
            bus.arm7.extkeyin.set(*button, true);
          } else if let Some(button) = self.button_map.get(&button) {
            bus.key_input_register.set(*button, true);
          }
        }
        _ => ()
      }
    }
  }

  pub fn render(&mut self, gpu: &mut GPU) {
    let creator = self.canvas.texture_creator();
    let mut texture_a = creator
      .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
      .unwrap();

    let mut texture_b = creator
      .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
      .unwrap();

    let engine_a = gpu.engine_a.lock().unwrap();
    let engine_b = gpu.engine_b.lock().unwrap();

    if gpu.powcnt1.contains(PowerControlRegister1::TOP_A) {
      texture_a.update(None, &engine_a.pixels, SCREEN_WIDTH as usize * 3).unwrap();
      texture_b.update(None, &engine_b.pixels, SCREEN_WIDTH as usize * 3).unwrap();
    } else {
      texture_a.update(None, &engine_b.pixels, SCREEN_WIDTH as usize * 3).unwrap();
      texture_b.update(None, &engine_a.pixels, SCREEN_WIDTH as usize * 3).unwrap();
    }


    let screen_a = Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let screen_b = Rect::new(0, SCREEN_HEIGHT as i32, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);

    self.canvas.copy(&texture_a, None, screen_a).unwrap();
    self.canvas.copy(&texture_b, None, screen_b).unwrap();

    self.canvas.present();
  }
}