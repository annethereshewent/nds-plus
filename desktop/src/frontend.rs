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
use glow::RGBA;
use imgui::{Context, Textures};
use imgui_glow_renderer::{
  glow::{
    HasContext, NativeTexture, PixelUnpackData, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, NEAREST, READ_FRAMEBUFFER, RGBA8, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, UNSIGNED_BYTE
  },
  Renderer
};
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
  audio::{
    AudioCallback,
    AudioDevice,
    AudioSpecDesired
  },
  controller::{
    Axis,
    Button,
    GameController
  },
  event::Event,
  keyboard::Keycode,
  video::{
    GLContext, GLProfile, Window
  },
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
  _controller: Option<GameController>,
  button_map: HashMap<Button, KeyInputRegister>,
  ext_button_map: HashMap<Button, ExternalKeyInputRegister>,
  ext_key_map: HashMap<Keycode, ExternalKeyInputRegister>,
  key_map: HashMap<Keycode, KeyInputRegister>,
  device: AudioDevice<DsAudioCallback>,
  use_control_stick: bool,
  controller_x: i16,
  controller_y: i16,
  renderer: Renderer,
  gl: imgui_glow_renderer::glow::Context,
  texture: NativeTexture,
  platform: SdlPlatform,
  email: String,
  password: String,
  show_menu: bool,
  imgui: imgui::Context,
  window: Window,
  textures: Textures<NativeTexture>,
  _gl_context: GLContext
}

impl Frontend {
  pub fn new(sdl_context: &Sdl, audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
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

    let gl = Self::glow_context(&window);

    let texture = unsafe { gl.create_texture().unwrap() };
    let framebuffer = unsafe { gl.create_framebuffer().unwrap() };

    unsafe {
      gl.bind_texture(TEXTURE_2D, Some(texture));

      gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
      gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

      gl.tex_storage_2d(
        TEXTURE_2D,
        1,
        RGBA8,
        SCREEN_WIDTH as i32 * 2,
        SCREEN_HEIGHT as i32 * 4
      );

      gl.bind_framebuffer(READ_FRAMEBUFFER, Some(framebuffer));
      gl.framebuffer_texture_2d(
        READ_FRAMEBUFFER,
        COLOR_ATTACHMENT0,
        TEXTURE_2D,
        Some(texture),
        0
      );

      gl.clear_color(0.0, 0.0, 0.0, 0.0);
    }

    let mut imgui = Context::create();

    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    imgui
      .fonts()
      .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let platform = SdlPlatform::init(&mut imgui);

    let mut textures = Textures::<NativeTexture>::new();

    let renderer = Renderer::initialize(
      &gl,
      &mut imgui,
      &mut textures,
      false
    ).unwrap();

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
      window,
      _controller,
      button_map,
      ext_button_map,
      key_map,
      ext_key_map,
      device,
      use_control_stick: false,
      controller_x: 0,
      controller_y: 0,
      email: String::new(),
      password: String::new(),
      show_menu: true,
      renderer,
      gl,
      texture,
      platform,
      imgui,
      textures,
      _gl_context: gl_context
    }
  }

  fn glow_context(window: &Window) -> imgui_glow_renderer::glow::Context {
    unsafe {
      imgui_glow_renderer::glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
  }

  pub fn handle_touchscreen(&mut self, bus: &mut Bus) {
    if !self.use_control_stick {
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
  }

  pub fn handle_events(&mut self, bus: &mut Bus) {
    for event in self.event_pump.poll_iter() {
      self.platform.handle_event(&mut self.imgui, &event);
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
            bus.gpu.engine_a.debug_on = !bus.gpu.engine_a.debug_on;
            bus.gpu.engine_b.debug_on = !bus.gpu.engine_b.debug_on;
            bus.gpu.engine3d.debug_on = !bus.gpu.engine3d.debug_on;
          } else if keycode.unwrap() == Keycode::T {
            self.use_control_stick = !self.use_control_stick;
            bus.arm7.extkeyin.set(ExternalKeyInputRegister::PEN_DOWN, !self.use_control_stick);
          } else if keycode.unwrap() == Keycode::E {
            bus.gpu.engine3d.current_polygon -= 1;
          } else if keycode.unwrap() == Keycode::R {
            bus.gpu.engine3d.current_polygon += 1;
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
          } else if button == Button::RightStick {
            self.use_control_stick = !self.use_control_stick;
            if self.use_control_stick {
              bus.arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
            } else {
              bus.arm7.extkeyin.insert(ExternalKeyInputRegister::PEN_DOWN);
            }
          }
        }
        Event::ControllerButtonUp { button, .. } => {
          if let Some(button) = self.ext_button_map.get(&button) {
            bus.arm7.extkeyin.set(*button, true);
          } else if let Some(button) = self.button_map.get(&button) {
            bus.key_input_register.set(*button, true);
          }
        }
        Event::ControllerAxisMotion { axis, value, .. } => {
          match axis {
            Axis::LeftX => {
              self.controller_x = value;
            }
            Axis::LeftY => {
              self.controller_y = value;
            }
            _ => ()
          }
        }
        _ => ()
      }
      if self.use_control_stick {
        bus.touchscreen.touch_screen_controller(self.controller_x, self.controller_y);
      }

    }
  }

  pub fn render(&mut self, gpu: &mut GPU) {

    let (top, bottom) = if gpu.powcnt1.contains(PowerControlRegister1::TOP_A) {
      (&gpu.engine_a.pixels, &gpu.engine_b.pixels)
    } else {
      (&gpu.engine_b.pixels, &gpu.engine_a.pixels)
    };

    unsafe {
      self.gl.clear(glow::COLOR_BUFFER_BIT);
      self.gl.bind_texture(TEXTURE_2D, Some(self.texture));

      self.gl.tex_sub_image_2d(
        TEXTURE_2D,
        0,
        0,
        0,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
        RGBA,
        UNSIGNED_BYTE,
        PixelUnpackData::Slice(&top)
      );

      self.gl.tex_sub_image_2d(
        TEXTURE_2D,
        0,
        0,
        SCREEN_HEIGHT as i32,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
        RGBA,
        UNSIGNED_BYTE,
        PixelUnpackData::Slice(&bottom)
      );

      self.gl.blit_framebuffer(
        0,
        SCREEN_HEIGHT as i32 * 2,
        SCREEN_WIDTH as i32,
        0,
        0,
        0,
        SCREEN_WIDTH as i32 * 2,
        SCREEN_HEIGHT as i32 * 4,
        COLOR_BUFFER_BIT,
        NEAREST
      );
    }
  }


  pub fn end_frame(&mut self) {
    self.window.gl_swap_window();
  }

  // TODO: move this back to frontend struct
  pub fn render_ui(&mut self) {
    self.platform.prepare_frame(&mut self.imgui, &mut self.window, &self.event_pump);

    let ui = self.imgui.new_frame();

    let mut show_popup = false;

    if let Some(token) = ui.begin_popup("cloud_modal") {
      ui.text("Please enter your credentials");

      ui.separator();

      ui.input_text("E-mail", &mut self.email).build();
      ui.input_text("Password", &mut self.password).password(true).build();

      ui.separator();

      if ui.button("Ok") && self.email != "" && self.password != "" {
        // need to log in to gooself.gle cloud!
        println!("loggin in with credentials {} and {}", self.email, self.password);
      }

      ui.same_line();

      if ui.button("Cancel") {
        ui.close_current_popup();
      }


      token.end();
    }

    if self.show_menu {
      ui.main_menu_bar(|| {
        if let Some(menu) = ui.begin_menu("File") {
          if ui.menu_item("Open") {

          }
          if ui.menu_item("Reset") {

          }
          if ui.menu_item("Cloud saves") {
            show_popup = true;

          }
          menu.end();
        }
      });
    }

    if show_popup {
      ui.open_popup("cloud_modal");
    }

    let draw_data = self.imgui.render();

    self.renderer.render(&self.gl, &mut self.textures, draw_data).unwrap();
  }
}