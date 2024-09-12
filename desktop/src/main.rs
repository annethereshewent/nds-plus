use std::{
  collections::VecDeque,
  env,
  fs::{
    self,
  },
  path::Path,
  sync::{
    Arc,
    Mutex
  }
};

use ds_emulator::nds::Nds;

use frontend::Frontend;

extern crate ds_emulator;

pub mod frontend;

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