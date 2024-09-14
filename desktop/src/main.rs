use std::{
  collections::VecDeque, env, fs::{
    self,
  },
  path::Path, sync::{
    Arc,
    Mutex
  }, time::{SystemTime, UNIX_EPOCH},
};

use ds_emulator::{cpu::bus::cartridge::BackupType, nds::Nds};

use frontend::{Frontend, UIAction};

extern crate ds_emulator;

pub mod frontend;
pub mod cloud_service;

fn detect_backup_type(frontend: &mut Frontend, nds: &mut Nds, rom_path: String, bytes: Option<Vec<u8>>) {
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

  let sdl_context = sdl2::init().unwrap();

  let mut frontend = Frontend::new(&sdl_context, audio_buffer.clone());

  let mut nds = Nds::new(
    Some(args[1].to_string()),
    Some(firmware_path.to_path_buf()),
    None,
    bios7_bytes,
    bios9_bytes,
    rom_bytes,
    skip_bios,
    audio_buffer
  );

  detect_backup_type(&mut frontend, &mut nds, args[1].to_string(), None);

  let mut frame_finished = false;

  let mut logged_in = frontend.cloud_service.lock().unwrap().logged_in;
  let mut has_backup = {
    match &nds.bus.borrow().cartridge.backup {
      BackupType::Eeprom(_) | BackupType::Flash(_) => true,
      BackupType::None => false
    }
  };

  let mut rom_path = args[1].to_string();

  loop {
    while !frame_finished {
      frame_finished = nds.step();
    }

    // some more hacky shit because rust is a fucking asshole
    {
      let ref mut bus = *nds.bus.borrow_mut();

      bus.gpu.frame_finished = false;
      bus.gpu.cap_fps();

      frame_finished = false;

      frontend.render(&mut bus.gpu);
    }

    match frontend.render_ui() {
      UIAction::None => (),
      UIAction::LoadGame(path) => {
        rom_path = path.clone().to_string_lossy().to_string();
        nds.load_game(path);
        nds.reset();
        detect_backup_type(&mut frontend, &mut nds, rom_path.clone(), None);

        has_backup = {
          match &nds.bus.borrow().cartridge.backup {
            BackupType::Eeprom(_) | BackupType::Flash(_) => true,
            BackupType::None => false
          }
        };

        continue;
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

        nds.reset();

        logged_in = frontend.cloud_service.lock().unwrap().logged_in;

        detect_backup_type(&mut frontend, &mut nds, rom_path.clone(), bytes);

        has_backup = {
          match &nds.bus.borrow().cartridge.backup {
            BackupType::Eeprom(_) | BackupType::Flash(_) => true,
            BackupType::None => false
          }
        };

        continue;
      }
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
  }
}