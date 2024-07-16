use std::{cell::RefCell, collections::{HashMap, VecDeque}, ops::DerefMut, rc::Rc, sync::{Arc, Mutex}};

use ds_emulator::{cpu::{bus::Bus, registers::key_input_register::KeyInputRegister}, gpu::{GPU, SCREEN_HEIGHT, SCREEN_WIDTH}, nds::Nds};
use sdl2::{audio::{AudioCallback, AudioDevice, AudioSpecDesired}, controller::{Button, GameController}, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, render::Canvas, video::Window, EventPump, Sdl};

struct DsAudioCallback {
  audio_samples: Arc<Mutex<VecDeque<f32>>>
}

impl AudioCallback for DsAudioCallback {
  type Channel = f32;

  fn callback(&mut self, buf: &mut [Self::Channel]) {
    let mut audio_samples = self.audio_samples.lock().unwrap();
    let len = audio_samples.len();

    let (last_left, last_right) = if len > 1 {
      (audio_samples[len - 2], audio_samples[len - 1])
    } else {
      (0.0, 0.0)
    };

    let mut index = 0;

    for b in buf.iter_mut() {
      *b = if let Some(sample) = audio_samples.pop_front() {
        sample
      } else {
        if  index % 2 == 0 { last_left } else { last_right }
      };

      index += 1;
    }
  }
}

pub struct Frontend {
  event_pump: EventPump,
  canvas: Canvas<Window>,
  _controller: Option<GameController>,
  button_map: HashMap<Button, KeyInputRegister>,
  key_map: HashMap<Keycode, KeyInputRegister>,
  device: AudioDevice<DsAudioCallback>
}

impl Frontend {
  pub fn new(sdl_context: &Sdl, audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
      .window("DS Emulator", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3 * 2) as u32)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let creator = canvas.texture_creator();

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
      freq: Some(32768),
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

    let mut button_map = HashMap::new();

    Self {
      event_pump,
      canvas,
      _controller,
      button_map,
      key_map,
      device
    }
  }

  pub fn handle_events(&mut self, bus: &mut Bus) {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => std::process::exit(0),
        Event::KeyDown { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            bus.key_input_register.set(*button, false);
          } else if keycode.unwrap() == Keycode::G {
            bus.debug_on = !bus.debug_on
          } else if keycode.unwrap() == Keycode::F {
            bus.gpu.engine_a.debug_on = !bus.gpu.engine_a.debug_on;
            bus.gpu.engine_b.debug_on = !bus.gpu.engine_b.debug_on;
          }
        }
        Event::KeyUp { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
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

    texture_a.update(None, &gpu.engine_a.pixels, SCREEN_WIDTH as usize * 3).unwrap();
    texture_b.update(None, &gpu.engine_b.pixels, SCREEN_WIDTH as usize * 3).unwrap();

    let screen_a = Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let screen_b = Rect::new(0, SCREEN_HEIGHT as i32, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);

    self.canvas.copy(&texture_a, None, screen_a).unwrap();
    self.canvas.copy(&texture_b, None, screen_b).unwrap();

    self.canvas.present();
  }
}