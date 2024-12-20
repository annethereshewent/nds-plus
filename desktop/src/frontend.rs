use std::{
 collections::{
    HashMap,
    VecDeque
  }, fs, path::{Path, PathBuf}, sync::{
    Arc,
    Mutex
  }, time::{SystemTime, UNIX_EPOCH}
};

use chrono::DateTime;
use directories::UserDirs;
use ds_emulator::{
  apu::Sample,
  cpu::{
    bus::{cartridge::BackupType, spi::SPI, touchscreen::SAMPLE_SIZE, Bus},
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
  }, nds::Nds
};

use glow::RGBA;
use imgui::{Context, Textures};
use imgui_glow_renderer::{
  glow::{
    HasContext,
    NativeTexture,
    PixelUnpackData,
    COLOR_ATTACHMENT0,
    COLOR_BUFFER_BIT,
    NEAREST,
    READ_FRAMEBUFFER,
    RGBA8,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    UNSIGNED_BYTE
  },
  Renderer
};
use imgui_sdl2_support::SdlPlatform;
use native_dialog::FileDialog;
use sdl2::{
  audio::{
    AudioCallback,
    AudioDevice,
    AudioSpecDesired
  }, controller::{
    Axis,
    Button,
    GameController
  }, event::Event, keyboard::Keycode, video::{
    GLContext, GLProfile, Window
  }, AudioSubsystem, EventPump, Sdl
};

use crate::cloud_service::CloudService;

pub enum UIAction {
  None,
  Reset(bool),
  LoadGame(PathBuf),
  CreateSaveState,
  LoadSaveState(PathBuf)
}

pub struct DsAudioCallback {
  pub audio_samples: Arc<Mutex<VecDeque<f32>>>
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

pub struct DsAudioRecording {
  pub mic_samples: Arc<Mutex<Box<[i16]>>>,
  index: usize
}

impl AudioCallback for DsAudioRecording {
  type Channel = i16;

  fn callback(&mut self, buf: &mut [Self::Channel]) {
    let mut mic_samples = self.mic_samples.lock().unwrap();

    if self.index + buf.len() > mic_samples.len() {
      self.index = 0;
    }

    for b in buf {
      mic_samples[self.index] = *b;

      self.index += 1;
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
  pub device: AudioDevice<DsAudioCallback>,
  use_control_stick: bool,
  controller_x: i16,
  controller_y: i16,
  renderer: Renderer,
  gl: imgui_glow_renderer::glow::Context,
  texture: NativeTexture,
  platform: SdlPlatform,
  show_menu: bool,
  imgui: imgui::Context,
  window: Window,
  textures: Textures<NativeTexture>,
  _gl_context: GLContext,
  pub cloud_service: Arc<Mutex<CloudService>>,
  pub capture_device: Option<AudioDevice<DsAudioRecording>>,
  audio_subsystem: AudioSubsystem,
  mic_samples: Arc<Mutex<Box<[i16]>>>,
  pub rom_loaded: bool,
  pub rom_path: String,
  bios7_file: String,
  bios9_file: String,
  firmware: PathBuf,
  pub has_backup: bool,
  pub save_entries: Vec<String>
}

impl Frontend {
  pub fn new(
    sdl_context: &Sdl,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    mic_samples: Arc<Mutex<Box<[i16]>>>,
    rom_path: String,
    bios7_file: String,
    bios9_file: String,
    firmware: PathBuf,
    save_entries: Vec<String>
  ) -> Self {
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    let window = video_subsystem
      .window("NDS Plus", SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 4)
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

    key_map.insert(Keycode::L, KeyInputRegister::ButtonA);


    key_map.insert(Keycode::K, KeyInputRegister::ButtonB);

    key_map.insert(Keycode::C, KeyInputRegister::ButtonL);
    key_map.insert(Keycode::V, KeyInputRegister::ButtonR);

    key_map.insert(Keycode::Return, KeyInputRegister::Start);
    key_map.insert(Keycode::Tab, KeyInputRegister::Select);

    let mut ext_key_map = HashMap::new();

    ext_key_map.insert(Keycode::J, ExternalKeyInputRegister::BUTTON_Y);
    ext_key_map.insert(Keycode::I, ExternalKeyInputRegister::BUTTON_X);

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
      show_menu: true,
      renderer,
      gl,
      texture,
      platform,
      imgui,
      textures,
      _gl_context: gl_context,
      cloud_service: Arc::new(Mutex::new(CloudService::new())),
      capture_device: None,
      audio_subsystem,
      mic_samples: mic_samples.clone(),
      rom_loaded: false,
      rom_path,
      bios7_file,
      bios9_file,
      has_backup: false,
      firmware,
      save_entries
    }
  }

  pub fn start_mic(&mut self) {
    let capture_spec = AudioSpecDesired {
      freq: Some(44100),
      channels: Some(1),
      samples: Some(1024)
    };

    let capture_device = match self.audio_subsystem.open_capture(None, &capture_spec, |_| {
      DsAudioRecording {
        mic_samples: self.mic_samples.clone(),
        index: 0
      }
    }) {
      Ok(capture) => {
        Some(capture)
      }
      Err(_) => None
    };

    self.capture_device = capture_device;

    if let Some(ref capture) = self.capture_device {
      capture.resume();
    }
  }

  fn glow_context(window: &Window) -> imgui_glow_renderer::glow::Context {
    unsafe {
      imgui_glow_renderer::glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
  }

  pub fn resume_mic(&mut self) {
    if let Some(ref capture) = self.capture_device {
      capture.pause();
      capture.resume();
    }
  }

  pub fn handle_touchscreen(&mut self, nds: &mut Nds) {
    let ref mut bus = *nds.bus.borrow_mut();
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

  pub fn clear_framebuffer(&mut self) {
    unsafe {
      self.gl.clear(glow::COLOR_BUFFER_BIT);
    }
  }

  pub fn handle_romless_events(&mut self) {
    for event in self.event_pump.poll_iter() {
      self.platform.handle_event(&mut self.imgui, &event);
      match event {
        Event::Quit { .. } => std::process::exit(0),
        Event::KeyDown { keycode, .. } => {
          if keycode.unwrap() == Keycode::Escape {
            self.show_menu = !self.show_menu;
          }
        }
        _ => ()
      }
    }
  }

  pub fn create_save_state(nds: &mut Nds, rom_path: String, state_paths: &mut Vec<String>, quick_save: bool) {
    if let Some(user_dirs) = UserDirs::new() {
      let document_path = user_dirs.document_dir().unwrap();

      // first check to see if any save states exist
      let delimiter = if std::env::consts::OS == "windows" {
        "\\"
      } else {
        "/"
      };

      let folder_path = Path::new(
        &format!(
          "{}{delimiter}NDS-Plus{delimiter}save_states{delimiter}{}",
          document_path.to_str().unwrap(),
          rom_path.split(delimiter).last().unwrap()
        )
      ).with_extension("");

      let (save_name, state_path) = if quick_save {
        (format!("{}{delimiter}save_1.state", folder_path.to_str().unwrap()), "save_1.state".to_string())
      } else {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        let filename = format!("save_{}.state", current_time);

        (format!("{}{delimiter}{}", folder_path.to_str().unwrap(), filename), filename)
      };

      if !quick_save {
        state_paths.push(state_path);
      }
      let buf = nds.create_save_state();

      std::thread::spawn(move || {
        // compressing generally takes the longest, so punting it to a thread should save some time
        let compressed = zstd::encode_all(&*buf, 9).unwrap();

        fs::write(save_name, compressed).unwrap();

        println!("finished creating save state");
      });
    }


  }

  pub fn load_save_state(
    nds: &mut Nds,
    bios7_file: String,
    bios9_file: String,
    rom_path: String,
    firmware: &PathBuf,
    device: &mut AudioDevice<DsAudioCallback>,
    capture_device: Option<&mut AudioDevice<DsAudioRecording>>,
    logged_in: bool,
    has_backup: bool,
    state_path: PathBuf
  ) {
    let rom_path = Path::new(&rom_path);

    let mut buf = match fs::read(state_path) {
      Ok(bytes) => bytes,
      Err(_) => return
    };

    buf = zstd::decode_all(&*buf).unwrap();

    nds.load_save_state(&buf);

    // reload bioses, firmware, and rom
    {
      let ref mut bus = *nds.bus.borrow_mut();
      bus.arm7.bios7 = fs::read(bios7_file).unwrap();

      bus.arm9.bios9 = fs::read(bios9_file).unwrap();

      bus.cartridge.rom = fs::read(rom_path).unwrap();

      let backup_file = Bus::load_firmware(Some(firmware.to_path_buf()), None);

      bus.spi = SPI::new(backup_file);

      // recreate mic and audio buffers
      bus.touchscreen.mic_buffer = vec![0; SAMPLE_SIZE].into_boxed_slice();

      if let Some(device) = capture_device {
        let samples = device.lock().mic_samples.clone();

        nds.mic_samples = samples;
      }

      let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

      bus.arm7.apu.audio_buffer = audio_buffer.clone();

      device.lock().audio_samples = audio_buffer.clone();

      if !logged_in && has_backup {
        let save_path = Path::new(&rom_path).with_extension("sav");
        match &mut bus.cartridge.backup {
          BackupType::Eeprom(eeprom) => {
            eeprom.backup_file.file = Some(fs::OpenOptions::new()
              .read(true)
              .write(true)
              .open(save_path)
              .unwrap());
          }
          BackupType::Flash(flash) => {
            flash.backup_file.file = Some(fs::OpenOptions::new()
              .read(true)
              .write(true)
              .open(save_path)
              .unwrap());
          }
          BackupType::None => unreachable!()
        }
      }
    }

    nds.arm7_cpu.bus = nds.bus.clone();
    nds.arm9_cpu.bus = nds.bus.clone();

    // repopulate arm and thumb luts
    nds.arm7_cpu.populate_arm_lut();
    nds.arm9_cpu.populate_arm_lut();

    nds.arm7_cpu.populate_thumb_lut();
    nds.arm9_cpu.populate_thumb_lut();

    nds.arm7_cpu.populate_arm_disassembly_lut();
    nds.arm9_cpu.populate_arm_disassembly_lut();

    nds.arm7_cpu.populate_thumb_disassembly_lut();
    nds.arm9_cpu.populate_thumb_disassembly_lut();
  }

  pub fn handle_events(&mut self, nds: &mut Nds) {
    // NOTE: i have to repeat this line like a million times:
    // let ref mut bus = *nds.bus.borrow_mut();
    // because if i don't rust will complain as usual about some stupid bullshit
    // here's another fuck you to you, rust.
    for event in self.event_pump.poll_iter() {
      self.platform.handle_event(&mut self.imgui, &event);
      match event {
        Event::Quit { .. } => std::process::exit(0),
        Event::KeyDown { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            let ref mut bus = *nds.bus.borrow_mut();
            self.show_menu = false;
            bus.key_input_register.set(*button, false);
          } else if let Some(button) = self.ext_key_map.get(&keycode.unwrap()) {
            let ref mut bus = *nds.bus.borrow_mut();
            self.show_menu = false;
            bus.arm7.extkeyin.set(*button, false);
          } else if keycode.unwrap() == Keycode::G {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.debug_on = !bus.debug_on;
          } else if keycode.unwrap() == Keycode::F {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.gpu.engine_a.debug_on = !bus.gpu.engine_a.debug_on;
            bus.gpu.engine_b.debug_on = !bus.gpu.engine_b.debug_on;
            bus.gpu.engine3d.debug_on = !bus.gpu.engine3d.debug_on;
          } else if keycode.unwrap() == Keycode::T {
            let ref mut bus = *nds.bus.borrow_mut();
            self.use_control_stick = !self.use_control_stick;
            bus.arm7.extkeyin.set(ExternalKeyInputRegister::PEN_DOWN, !self.use_control_stick);
          } else if keycode.unwrap() == Keycode::Escape {
            self.show_menu = !self.show_menu;
          } else if keycode.unwrap() == Keycode::F5 && self.rom_loaded {
            Self::create_save_state(
              nds,
              self.rom_path.clone(),
              &mut self.save_entries,
              true
            );
          } else if keycode.unwrap() == Keycode::F7 && self.rom_loaded {
            let user_dirs = UserDirs::new().unwrap();
            let documents_dir = user_dirs.document_dir().unwrap();
            let delimiter = if std::env::consts::OS == "windows" {
              "\\"
            } else {
              "/"
            };

            let path = Path::new(self.rom_path.split(delimiter).last().unwrap()).with_extension("");



            let state_path = Path::new(
              &format!(
                "{}{delimiter}NDS-Plus{delimiter}save_states{delimiter}{}{delimiter}save_1.state",
                documents_dir.to_str().unwrap(),
                path.to_str().unwrap()
              )
            ).to_path_buf();

            Self::load_save_state(
              nds,
              self.bios7_file.clone(),
              self.bios9_file.clone(),
              self.rom_path.clone(),
              &self.firmware,
              &mut self.device,
              self.capture_device.as_mut(),
              self.cloud_service.lock().unwrap().logged_in,
              self.has_backup,
              state_path
            );
          } else if keycode.unwrap() == Keycode::H {
            if !nds.stepping {
              nds.stepping = true;
              nds.paused = true
            } else {
              nds.stepping = false;
              nds.paused = false;
            }
          } else if keycode.unwrap() == Keycode::Y {
            nds.paused = !nds.paused;
          }
        }
        Event::KeyUp { keycode, .. } => {
          if let Some(button) = self.key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.key_input_register.set(*button, true);
          } else if let Some(button) = self.ext_key_map.get(&keycode.unwrap()) {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.arm7.extkeyin.set(*button, true);
          }
        }
        Event::ControllerButtonDown { button, .. } => {
          self.show_menu = false;
          if let Some(button) = self.ext_button_map.get(&button) {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.arm7.extkeyin.set(*button, false);
          } else if let Some(button) = self.button_map.get(&button) {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.key_input_register.set(*button, false);
          } else if button == Button::RightStick {
            self.use_control_stick = !self.use_control_stick;
            if self.use_control_stick {
              let ref mut bus = *nds.bus.borrow_mut();
              bus.arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
            } else {
              let ref mut bus = *nds.bus.borrow_mut();
              bus.arm7.extkeyin.insert(ExternalKeyInputRegister::PEN_DOWN);
            }
          }
        }
        Event::ControllerButtonUp { button, .. } => {
          if let Some(button) = self.ext_button_map.get(&button) {
            let ref mut bus = *nds.bus.borrow_mut();
            bus.arm7.extkeyin.set(*button, true);
          } else if let Some(button) = self.button_map.get(&button) {
            let ref mut bus = *nds.bus.borrow_mut();
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
            Axis::TriggerLeft => {
              if value == 0x7fff {

                Self::create_save_state(
                  nds,
                  self.rom_path.clone(),
                  &mut self.save_entries,
                  true
                );
              };
            }
            Axis::TriggerRight => {
              if value == 0x7fff {
                let user_dirs = UserDirs::new().unwrap();
                let documents_dir = user_dirs.document_dir().unwrap();

                let delimiter = if std::env::consts::OS == "windows" {
                  "\\"
                } else {
                  "/"
                };

                let path = Path::new(self.rom_path.split(delimiter).last().unwrap()).with_extension("");
                let state_path = Path::new(
                  &format!(
                    "{}{delimiter}NDS-Plus{delimiter}save_states{delimiter}{}{delimiter}save_1.state",
                    documents_dir.to_str().unwrap(),
                    path.to_str().unwrap()
                  )
                ).to_path_buf();

                Self::load_save_state(
                  nds,
                  self.bios7_file.clone(),
                  self.bios9_file.clone(),
                  self.rom_path.clone(),
                  &self.firmware,
                  &mut self.device,
                  self.capture_device.as_mut(),
                  self.cloud_service.lock().unwrap().logged_in,
                  self.has_backup,
                  state_path
                );
              }
            }
            _ => ()
          }
        }
        _ => ()
      }
      if self.use_control_stick {
        let ref mut bus = *nds.bus.borrow_mut();
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

  pub fn render_ui(&mut self) -> UIAction {
    self.platform.prepare_frame(&mut self.imgui, &mut self.window, &self.event_pump);

    let ui = self.imgui.new_frame();

    let mut action = UIAction::None;

    if self.show_menu {
      ui.main_menu_bar(|| {
        if let Some(menu) = ui.begin_menu("File") {
          if ui.menu_item("Open") {
            match FileDialog::new()
              .add_filter("NDS Rom file", &["nds"])
              .show_open_single_file() {
                Ok(path) => if let Some(path) = path {
                  action = UIAction::LoadGame(path);
                }
                Err(_) => ()
              }
          }
          if self.rom_loaded && ui.menu_item("Reset") {
            action = UIAction::Reset(true);
          }
          if ui.menu_item("Quit") {
            std::process::exit(0);
          }

          menu.end();
        }
        if self.rom_loaded {
          if let Some(menu) = ui.begin_menu("Save states") {
            if ui.menu_item("Create save state") {
              action = UIAction::CreateSaveState;
            }
            if let Some(menu) = ui.begin_menu("Load save state") {
              for entry in &self.save_entries {
                let mut save_name_str = entry.split("/").last().unwrap().to_string();

                let save_name = if save_name_str == "save_1.state" {
                  "Quick save".to_string()
                } else {
                  save_name_str = save_name_str.replace("save_", "");
                  save_name_str = save_name_str.replace(".state", "");

                  let timestamp = save_name_str.parse::<i64>().unwrap();
                  let date = DateTime::from_timestamp_millis(timestamp).unwrap();

                  format!("Save on {}", date.format("%m-%d-%Y %H:%M:%S"))
                };

                if ui.menu_item(save_name) {
                  let delimiter = if std::env::consts::OS == "windows" {
                    "\\"
                  } else {
                    "/"
                  };
                  let folder_path = Path::new(self.rom_path.split(delimiter).last().unwrap()).with_extension("");

                  let user_dirs = UserDirs::new().unwrap();
                  let documents_dir = user_dirs.document_dir().unwrap();

                  let save_entry = format!(
                    "{}{delimiter}NDS-Plus{delimiter}save_states{delimiter}{}{delimiter}{}",
                    documents_dir.to_str().unwrap(),
                    folder_path.to_str().unwrap(),
                    entry
                  );

                  action = UIAction::LoadSaveState(Path::new(&save_entry).to_path_buf());
                }
              }
              menu.end();
            }
            if let Some(menu) = ui.begin_menu("Delete save state") {
              let mut delete_pos = -1;
              for entry in &self.save_entries {
                let mut save_name_str = entry.split("/").last().unwrap().to_string();

                let save_name = if save_name_str == "save_1.state" {
                  "Quick save".to_string()
                } else {
                  save_name_str = save_name_str.replace("save_", "");
                  save_name_str = save_name_str.replace(".state", "");

                  let timestamp = save_name_str.parse::<i64>().unwrap();
                  let date = DateTime::from_timestamp_millis(timestamp).unwrap();

                  format!("Save on {}", date.format("%m-%d-%Y %H:%M:%S"))
                };

                if ui.menu_item(save_name) {
                  let user_dirs = UserDirs::new().unwrap();
                  let documents_dir = user_dirs.document_dir().unwrap();
                  let delimiter = if std::env::consts::OS == "windows" {
                    "\\"
                  } else {
                    "/"
                  };

                  let folder_path = Path::new(self.rom_path.split(delimiter).last().unwrap()).with_extension("");

                  let save_entry = format!(
                    "{}{delimiter}NDS-Plus{delimiter}save_states{delimiter}{}{delimiter}{}",
                    documents_dir.to_str().unwrap(),
                    folder_path.to_str().unwrap(),
                    entry
                  );

                  fs::remove_file(save_entry).unwrap();

                  if let Some(pos) = self.save_entries.iter().position(|entry2| entry2.to_string() == *entry) {
                    delete_pos = pos as isize;
                  }
                }
              }

              if delete_pos != -1 {
                self.save_entries.remove(delete_pos as usize);
              }

              menu.end();
            }

            menu.end();
          }

        }
        if let Some(menu) = ui.begin_menu("Cloud saves") {
          let mut cloud_service = self.cloud_service.lock().unwrap();

          if !cloud_service.logged_in && ui.menu_item("Log in to Google Cloud") {
            if self.rom_loaded {
              action = UIAction::Reset(false);
            }
            cloud_service.login();
          } else if cloud_service.logged_in && ui.menu_item("Log out of Google Cloud") {
            if self.rom_loaded {
              action = UIAction::Reset(false);
            }
            cloud_service.logout();
          }

          menu.end();
        }
      });
    }

    let draw_data = self.imgui.render();

    self.renderer.render(&self.gl, &mut self.textures, draw_data).unwrap();

    action
  }
}