use std::{
  collections::VecDeque, env, fs::{
    self,
  },
  path::Path, sync::{
    Arc,
    Mutex
  }, time::{SystemTime, UNIX_EPOCH},
};

use ds_emulator::{cpu::bus::{backup_file::BackupFile, cartridge::BackupType, spi::SPI, touchscreen::SAMPLE_SIZE, Bus}, nds::Nds};

use frontend::{Frontend, UIAction};

extern crate ds_emulator;

pub mod frontend;
pub mod cloud_service;

fn detect_backup_type(
  frontend: &mut Frontend,
  nds: &mut Nds,
  rom_path: String,
  bytes: Option<Vec<u8>>
) {
  if frontend.cloud_service.lock().unwrap().logged_in {
    let ref mut bus = *nds.bus.borrow_mut();

    if let Some(entry) = bus.cartridge.detect_backup_type() {
      let save_path = Path::new(&rom_path).with_extension("sav");
      let game_name = save_path.to_str().unwrap();

      let game_name = if game_name.contains("/") {
        game_name.split("/").last().unwrap()
      } else if game_name.contains("\\") {
        game_name.split("\\").last().unwrap()
      } else {
        game_name
      };

      let bytes = if let Some(bytes) = bytes {
        bytes
      } else {
        frontend.cloud_service.lock().unwrap().get_save(game_name)
      };

      bus.cartridge.set_cloud_backup(bytes, entry);
    }
  } else {
    let ref mut bus = *nds.bus.borrow_mut();

    if let Some(entry) = bus.cartridge.detect_backup_type() {
      let path = Path::new(&rom_path).with_extension("sav");
      bus.cartridge.set_backup(path, entry);
    }
  }
}

fn handle_frontend(
  frontend: &mut Frontend,
  rom_path_str: &mut String,
  nds: &mut Nds,
  has_backup: &mut bool,
  logged_in: &mut bool,
  bios7_file: &str,
  bios9_file: &str,
  firmware: &Path
) -> bool {
  match frontend.render_ui() {
    UIAction::None => (),
    UIAction::LoadGame(path) => {
      *rom_path_str = path.clone().to_string_lossy().to_string();
      let rom = fs::read(rom_path_str.clone()).unwrap();
      nds.reset(&rom);
      detect_backup_type(frontend, nds, rom_path_str.clone(), None);

      *has_backup = {
        match &nds.bus.borrow().cartridge.backup {
          BackupType::Eeprom(_) | BackupType::Flash(_) => true,
          BackupType::None => false
        }
      };

      frontend.rom_loaded = true;

      return true;
    }
    UIAction::Reset(get_bytes) => {
      // this is so that it doesn't have to fetch the save from the cloud all over again, which adds considerable lag
      let bytes = if frontend.cloud_service.lock().unwrap().logged_in && get_bytes {
        let ref bus = *nds.bus.borrow();

        match &bus.cartridge.backup {
          BackupType::Eeprom(eeprom) => Some(eeprom.backup_file.buffer.clone()),
          BackupType::Flash(flash)=> Some(flash.backup_file.buffer.clone()),
          BackupType::None => None
        }
      } else {
        None
      };

      let rom = nds.bus.borrow().cartridge.rom.clone();
      nds.reset(&rom);

      *logged_in = frontend.cloud_service.lock().unwrap().logged_in;

      detect_backup_type(frontend, nds, rom_path_str.clone(), bytes);

      *has_backup = {
        match &nds.bus.borrow().cartridge.backup {
          BackupType::Eeprom(_) | BackupType::Flash(_) => true,
          BackupType::None => false
        }
      };

      frontend.rom_loaded = true;

      return true;
    }
    UIAction::CreateSaveState => {
      let buf = nds.create_save_state();

      let path = Path::new(&rom_path_str).with_extension("state");

      fs::write(path, buf).unwrap();
    }
    UIAction::LoadSaveState => {
      let state_path = Path::new(&rom_path_str).with_extension("state");

      let rom_path = Path::new(&rom_path_str);

      let buf = fs::read(state_path).unwrap();

      nds.load_save_state(&buf);

      // reload bioses, firmware, and rom
      let mut bytes: Option<Vec<u8>> = None;
      {
        let ref mut bus = *nds.bus.borrow_mut();
        bus.arm7.bios7 = fs::read(bios7_file).unwrap();

        bus.arm9.bios9 = fs::read(bios9_file).unwrap();

        bus.cartridge.rom = fs::read(rom_path).unwrap();

        let backup_file = Bus::load_firmware(Some(firmware.to_path_buf()), None);

        bus.spi = SPI::new(backup_file);

        // recreate mic and audio buffers
        bus.touchscreen.mic_buffer = vec![0; SAMPLE_SIZE].into_boxed_slice();

        if let Some(device) = &mut frontend.capture_device {
          let samples = device.lock().mic_samples.clone();

          nds.mic_samples = samples;
        }

        let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

        bus.arm7.apu.audio_buffer = audio_buffer.clone();

        frontend.device.lock().audio_samples = audio_buffer.clone();

        *logged_in = frontend.cloud_service.lock().unwrap().logged_in;



        if !*logged_in && *has_backup {
          let save_path = Path::new(&rom_path_str).with_extension("sav");
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

      return true;
    }
  }

  return false;
}

fn main() {
  let args: Vec<String> = env::args().collect();

  let mut base_index = 1;
  let mut rom_path = "".to_string();

  if args.len() >= base_index + 1 {
    if args[base_index] != "ignore-dir" {
      rom_path = args[base_index].to_string();
      base_index += 1;

      let exe_path = std::env::current_exe().unwrap();

      let exe_parent = exe_path.parent().unwrap();

      env::set_current_dir(exe_parent).unwrap();
    } else {
      // this option ignores setting the directory to the exe's folder directory.
      // this option should be used when running the program via "cargo run"
      base_index += 1;

      if args.len() >= base_index + 1 {
        rom_path = args[base_index].to_string();
        base_index += 1;
      }
    }
  } else {
    let exe_path = std::env::current_exe().unwrap();

    let exe_parent = exe_path.parent().unwrap();

    env::set_current_dir(exe_parent).unwrap();
  }

  let mut skip_bios = true;

  if args.len() == base_index + 1 && args[base_index] == "--start-bios" {
    skip_bios = false;
  }

  let audio_buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
  let mic_samples: Arc<Mutex<Box<[i16]>>> = Arc::new(Mutex::new(vec![0; 2048].into_boxed_slice()));

  let os_bios7_file = "./freebios/drastic_bios_arm7.bin";
  let os_bios9_file = "./freebios/drastic_bios_arm9.bin";

  let bios7_file = "./bios7.bin";
  let bios9_file = "./bios9.bin";
  let firmware_path = "./firmware.bin";

  let mut actual_bios7_file = bios7_file;
  let mut actual_bios9_file = bios9_file;

  let bios7_bytes = match fs::read(bios7_file) {
    Ok(bytes) => bytes,
    Err(_) => {
      actual_bios7_file = os_bios7_file;
      fs::read(os_bios7_file).unwrap()
    }
  };

  let bios9_bytes = match fs::read(bios9_file) {
    Ok(bytes) => bytes,
    Err(_) => {
      actual_bios9_file = os_bios9_file;
      fs::read(os_bios9_file).unwrap()
    }
  };

  let firmware_path = Path::new(firmware_path);

  let sdl_context = sdl2::init().unwrap();

  let mut frontend = Frontend::new(&sdl_context, audio_buffer.clone(), mic_samples.clone());

  let mut nds = Nds::new(
    Some(firmware_path.to_path_buf()),
    None,
    bios7_bytes,
    bios9_bytes,
    audio_buffer,
    mic_samples
  );

  let mut has_backup = false;
  if rom_path != "" {
    let rom_bytes = fs::read(&rom_path).unwrap();
    nds.init(&rom_bytes, skip_bios);

    frontend.rom_loaded = true;

    detect_backup_type(&mut frontend, &mut nds, rom_path.clone(), None);
  }

  let mut frame_finished = false;

  let mut logged_in = frontend.cloud_service.lock().unwrap().logged_in;
  if frontend.rom_loaded {
    has_backup = match &nds.bus.borrow().cartridge.backup {
      BackupType::Eeprom(_) | BackupType::Flash(_) => true,
      BackupType::None => false
    };

    frontend.start_mic();
  }

  loop {
    if frontend.rom_loaded {
      let frame_start = nds.arm7_cpu.cycles;

      while !frame_finished {
        frame_finished = nds.step();
        nds.bus.borrow_mut().frame_cycles = nds.arm7_cpu.cycles - frame_start;
      }

      // need to do this or else will rust complain about borrowing and ownership
      {
        let ref mut bus = *nds.bus.borrow_mut();

        bus.gpu.frame_finished = false;
        bus.gpu.cap_fps();

        let mic_samples = nds.mic_samples.lock().unwrap();

        bus.touchscreen.update_mic_buffer(&mic_samples.to_vec());

        frame_finished = false;

        frontend.render(&mut bus.gpu);
      }

      frontend.resume_mic();

      if handle_frontend(
        &mut frontend,
        &mut rom_path,
        &mut nds,
        &mut has_backup,
        &mut logged_in,
        &actual_bios7_file,
        &actual_bios9_file,
        &firmware_path
      ) {
        continue;
      }

      let ref mut bus = *nds.bus.borrow_mut();

      frontend.end_frame();

      frontend.handle_events(bus);
      frontend.handle_touchscreen(bus);
      if logged_in && has_backup {
        let file = match &mut bus.cartridge.backup {
          BackupType::Eeprom(eeprom) => &mut eeprom.backup_file,
          BackupType::Flash(flash) => &mut flash.backup_file,
          BackupType::None => unreachable!()
        };

        let current_time = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .expect("an error occurred")
          .as_millis();

        if file.last_write != 0 && current_time - file.last_write > 1000 {
          let cloud_service = frontend.cloud_service.clone();
          let bytes = file.buffer.clone();
          std::thread::spawn(move || {
            let mut cloud_service = cloud_service.lock().unwrap();
            println!("saving file....");
            cloud_service.upload_save(&bytes);
            println!("finished saving!");
          });
          file.last_write = 0;
        }
      }
    } else {
      frontend.clear_framebuffer();

      if handle_frontend(
        &mut frontend,
        &mut rom_path,
        &mut nds,
        &mut has_backup,
        &mut logged_in,
        &actual_bios7_file,
        &actual_bios9_file,
        &firmware_path
      ) {
        frontend.start_mic();
        continue;
      }

      frontend.end_frame();
      frontend.handle_romless_events();
    }
  }
}