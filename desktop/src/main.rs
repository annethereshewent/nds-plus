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
    GPU,
    SCREEN_HEIGHT,
    SCREEN_WIDTH
  },
  nds::Nds
};

use glow::{
  NativeTexture,
  PixelUnpackData,
  Texture,
  COLOR_ATTACHMENT0,
  DEBUG_OUTPUT,
  DEBUG_OUTPUT_SYNCHRONOUS,
  LINEAR,
  NEAREST,
  READ_FRAMEBUFFER,
  RGB8,
  RGBA,
  RGBA8,
  TEXTURE_2D,
  TEXTURE_MAG_FILTER,
  TEXTURE_MIN_FILTER, UNSIGNED_BYTE
};

use imgui::{
  Context,
  TextureId,
  Textures
};

use imgui_glow_renderer::{
  glow::{
    UNSIGNED_SHORT_1_5_5_5_REV
  },
  AutoRenderer,
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
  pixels::PixelFormatEnum,
  rect::Rect, video::{
    GLProfile,
    Window
  },
  EventPump
};
// use imgui_glow_renderer::glow::HasContext;
use glow::HasContext;

extern crate ds_emulator;


fn render(
  gpu: &mut GPU,
  //platform: &mut SdlPlatform,
  // renderer: &mut AutoRenderer,
  // imgui: &mut Context,
  window: &Window,
  texture: &mut u32
) {
  /*
                gl::GenFramebuffers(1, &mut fbo as *mut u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                screen_tex,
                0,
            );
  */

  unsafe {
    gl::BindTexture(TEXTURE_2D, *texture);

    gl::Clear(gl::COLOR_BUFFER_BIT);

    gl::TexSubImage2D(
      gl::TEXTURE_2D,
      0,
      0,
      0,
      SCREEN_WIDTH.into(),
      SCREEN_HEIGHT.into(),
      gl::RGBA,
      gl::UNSIGNED_SHORT_1_5_5_5_REV,
      gpu.engine_a.pixels.as_ptr() as *const std::ffi::c_void,
    );
  }

  // platform.prepare_frame(imgui, window, event_pump);

  // let ui = imgui.new_frame();

  // ui.show_demo_window(&mut true);

  // imgui::Image::new(texture_id, [SCREEN_WIDTH as f32 * 2.0, SCREEN_HEIGHT as f32 * 2.0]).build(ui);

  // let draw_data = imgui.render();

  // renderer.render(draw_data);

  window.gl_swap_window();
}

fn handle_events(
  bus: &mut Bus,
  event_pump: &mut EventPump,
  key_map: &HashMap<Keycode, KeyInputRegister>,
  ext_key_map: &HashMap<Keycode, ExternalKeyInputRegister>,
  button_map: &HashMap<Button, KeyInputRegister>,
  ext_button_map: &HashMap<Button, ExternalKeyInputRegister>,
  use_control_stick: &mut bool,
  controller_x: &mut i16,
  controller_y: &mut i16
) {
  for event in event_pump.poll_iter() {
    match event {
      Event::Quit { .. } => std::process::exit(0),
      Event::KeyDown { keycode, .. } => {
        if let Some(button) = key_map.get(&keycode.unwrap_or(Keycode::Return)) {
          bus.key_input_register.set(*button, false);
        } else if let Some(button) = ext_key_map.get(&keycode.unwrap()) {
          bus.arm7.extkeyin.set(*button, false);
        } else if keycode.unwrap() == Keycode::G {
          bus.debug_on = !bus.debug_on
        } else if keycode.unwrap() == Keycode::F {
          bus.gpu.engine_a.debug_on = !bus.gpu.engine_a.debug_on;
          bus.gpu.engine_b.debug_on = !bus.gpu.engine_b.debug_on;
          bus.gpu.engine3d.debug_on = !bus.gpu.engine3d.debug_on;
        } else if keycode.unwrap() == Keycode::T {
          *use_control_stick = !*use_control_stick;
          bus.arm7.extkeyin.set(ExternalKeyInputRegister::PEN_DOWN, !*use_control_stick);
        } else if keycode.unwrap() == Keycode::E {
          bus.gpu.engine3d.current_polygon -= 1;
        } else if keycode.unwrap() == Keycode::R {
          bus.gpu.engine3d.current_polygon += 1;
        }
      }
      Event::KeyUp { keycode, .. } => {
        if let Some(button) = key_map.get(&keycode.unwrap_or(Keycode::Return)) {
          bus.key_input_register.set(*button, true);
        } else if let Some(button) = ext_key_map.get(&keycode.unwrap()) {
          bus.arm7.extkeyin.set(*button, true);
        }
      }
      Event::ControllerButtonDown { button, .. } => {
        if let Some(button) = ext_button_map.get(&button) {
          bus.arm7.extkeyin.set(*button, false);
        } else if let Some(button) = button_map.get(&button) {
          bus.key_input_register.set(*button, false);
        } else if button == Button::RightStick {
          *use_control_stick = !*use_control_stick;
          if *use_control_stick {
            bus.arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
          } else {
            bus.arm7.extkeyin.insert(ExternalKeyInputRegister::PEN_DOWN);
          }
        }
      }
      Event::ControllerButtonUp { button, .. } => {
        if let Some(button) = ext_button_map.get(&button) {
          bus.arm7.extkeyin.set(*button, true);
        } else if let Some(button) = button_map.get(&button) {
          bus.key_input_register.set(*button, true);
        }
      }
      Event::ControllerAxisMotion { axis, value, .. } => {
        match axis {
          Axis::LeftX => {
            *controller_x = value;
          }
          Axis::LeftY => {
            *controller_y = value;
          }
          _ => ()
        }
      }
      _ => ()
    }
    if *use_control_stick {
      bus.touchscreen.touch_screen_controller(*controller_x, *controller_y);
    }
  }
}

fn handle_touchscreen(bus: &mut Bus, event_pump: &EventPump, use_control_stick: bool) {
  if !use_control_stick {
    let state = event_pump.mouse_state();

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
}

fn glow_context(window: &Window) -> imgui_glow_renderer::glow::Context {
  unsafe {
    imgui_glow_renderer::glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
  }
}

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

  let video_subsystem = sdl_context.video().unwrap();

  let gl_attr = video_subsystem.gl_attr();

  gl_attr.set_context_version(3, 2);
  gl_attr.set_context_profile(GLProfile::Core);

  let window = video_subsystem
    .window("DS Emulator", SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 4)
    .opengl()
    .position_centered()
    .build()
    .unwrap();

  let gl_context = window.gl_create_context().unwrap();

  window.gl_make_current(&gl_context).unwrap();

  window.subsystem().gl_set_swap_interval(1).unwrap();

  //let mut gl = glow_context(&window);
  gl::load_with(|s| window.subsystem().gl_get_proc_address(s) as *const _);

  // let program = unsafe { gl2.create_program().expect("Cannot create program") };

  // let texture = unsafe { gl2.create_texture().unwrap() };

  let mut texture = 0u32;

  let mut framebuffer = 0u32;

  unsafe {
    gl::GenTextures(1, &mut texture as *mut u32);
    gl::BindTexture(gl::TEXTURE_2D, texture);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexStorage2D(
        gl::TEXTURE_2D,
        1,
        gl::RGBA8,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
    );

    gl::GenFramebuffers(1, &mut framebuffer as *mut u32);
    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        texture,
        0,
    );
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
  }

  // let mut imgui = Context::create();

  // imgui.set_ini_filename(None);
  // imgui.set_log_filename(None);

  // imgui
  //   .fonts()
  //   .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

  // let mut platform = SdlPlatform::init(&mut imgui);
  // let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();

  // let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  // canvas.set_scale(2.0, 2.0).unwrap();

  let mut event_pump = sdl_context.event_pump().unwrap();

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

  let mut frame_finished = false;

  let mut use_control_stick = false;
  let mut controller_x = 0;
  let mut controller_y = 0;

  // let dummy = unsafe { gl.create_texture().unwrap() };

  // textures.insert(dummy);

  // let texture_id_a = textures.insert(dummy);
  // let texture_id_b = textures.insert(dummy);

  loop {
    while !frame_finished {
      frame_finished = nds.step();
    }

    let ref mut bus = *nds.bus.borrow_mut();

    bus.gpu.frame_finished = false;
    bus.gpu.cap_fps();

    frame_finished = false;

    // render stuff
    render(
      &mut bus.gpu,
      //&mut platform,
      // &mut renderer,
      // &mut imgui,
      &window,
      &mut texture
    );

    handle_events(
      bus, &mut event_pump,
      &key_map,
      &ext_key_map,
      &button_map,
      &ext_button_map,
      &mut use_control_stick,
      &mut controller_x,
      &mut controller_y,
    );

    handle_touchscreen(bus, &event_pump, use_control_stick);
  }
}