use std::{
  collections::{
    HashMap,
    VecDeque
  },
  env,
  fs::{
    self,
    File
  },
  path::Path,
  rc::Rc,
  sync::{
    Arc,
    Mutex
  }
};

use ds_emulator::{
  apu::{
    Sample,
    APU
  },
  cpu::{
    bus::Bus,
    registers::{
      external_key_input_register::ExternalKeyInputRegister,
      key_input_register::KeyInputRegister
    }
  },
  gpu::{
    registers::power_control_register1::PowerControlRegister1,
    GPU,
    SCREEN_HEIGHT,
    SCREEN_WIDTH
  },
  nds::Nds
};

use frontend::Frontend;
use glow::{
  COLOR_ATTACHMENT0,
  COLOR_BUFFER_BIT,
  NEAREST,
  READ_FRAMEBUFFER,
  RGBA,
  RGBA8,
  TEXTURE_2D,
  TEXTURE_MAG_FILTER,
  TEXTURE_MIN_FILTER,
  UNSIGNED_BYTE
};

use imgui::{
  Context,
  Textures
};

use imgui_glow_renderer::{
  glow::{
    HasContext,
    NativeTexture,
    PixelUnpackData
  },
  Renderer
};
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
  audio::{
    AudioCallback,
    AudioSpecDesired
  },
  controller::{
    Axis,
    Button
  },
  event::Event,
  keyboard::Keycode,
  video::{
    GLProfile,
    Window
  },
  EventPump
};

extern crate ds_emulator;

pub mod frontend;

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

fn gl_debug_callback(
  _source: u32,
  _type: u32,
  _id: u32,
  sev: u32,
  message: &str,
) {
  println!("{message}");
}

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    panic!("please specify a rom file.");
  }

  let mut skip_bios = true;

  if args.len() == 3 && args[2] == "--start-bios" {
    skip_bios = false;
  }

  let audio_buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));

  let bios7_file = "../bios7.bin";
  let bios9_file = "../bios9.bin";
  let firmware_path = "../firmware.bin";

  let bios7_bytes = fs::read(bios7_file).unwrap();
  let bios9_bytes = fs::read(bios9_file).unwrap();
  let rom_bytes = fs::read(&args[1]).unwrap();
  let firmware_path = Path::new(firmware_path);

  let mut nds = Nds::new(
    Some(args[1].to_string()),
    Some(firmware_path.to_path_buf()),
    None,
    bios7_bytes,
    bios9_bytes,
    rom_bytes,
    skip_bios,
    audio_buffer.clone()
  );

  let sdl_context = sdl2::init().unwrap();

  let mut frontend = Frontend::new(&sdl_context, audio_buffer.clone());


  let mut frame_finished = false;

  loop {
    while !frame_finished {
      frame_finished = nds.step();
    }

    let ref mut bus = *nds.bus.borrow_mut();

    bus.gpu.frame_finished = false;
    bus.gpu.cap_fps();

    frame_finished = false;

    frontend.render(&mut bus.gpu);
    frontend.render_ui();
    frontend.end_frame();

    frontend.handle_events(bus);
    frontend.handle_touchscreen(bus);

  }
}