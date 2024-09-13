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

use frontend::Frontend;

extern crate ds_emulator;

pub mod frontend;
pub mod cloud_service;


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
    audio_buffer.clone()
  );

  if frontend.cloud_service.logged_in {
     // fetch the game's save from the cloud

    // rust once again making me do really stupid fucking shit.
    // i have to do things in two steps (create path and save to variable, then convert to string)
    // as opposed to doing it all at once.
    let save_path = Path::new(&args[1]).with_extension("sav");
    let game_name = save_path.to_str().unwrap();

    let game_name = game_name.split("/").last().unwrap();

    let bytes = frontend.cloud_service.get_save(game_name);

    nds.bus.borrow_mut().cartridge.detect_cloud_backup_type(bytes);
  } else {
    let path = Path::new(&args[1]).with_extension("sav");

    nds.bus.borrow_mut().cartridge.detect_backup_type(path);

  }

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
    if frontend.cloud_service.logged_in {
      let has_backup = match &bus.cartridge.backup {
        BackupType::Eeprom(_) | BackupType::Flash(_) => true,
        BackupType::None => false
      };

      if has_backup {
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
          println!("ayyy im finally uploading the save!!!!");
          frontend.cloud_service.upload_save(&file.buffer);
          file.last_write = 0;
        }
      }
    }
  }
}