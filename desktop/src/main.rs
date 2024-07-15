use std::{collections::{HashMap, VecDeque}, env, fs};

use ds_emulator::{apu::APU, cpu::registers::key_input_register::KeyInputRegister, gpu::{SCREEN_HEIGHT, SCREEN_WIDTH}, nds::Nds};
use frontend::Frontend;
use sdl2::{audio::{AudioCallback, AudioSpecDesired}, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect};

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

  let bios7_file = "../bios7.bin";
  let bios9_file = "../bios9.bin";
  let firmware_file = "../firmware.bin";

  let bios7_bytes = fs::read(bios7_file).unwrap();
  let bios9_bytes = fs::read(bios9_file).unwrap();
  let rom_bytes = fs::read(&args[1]).unwrap();
  let firmware_bytes = fs::read(firmware_file).unwrap();

  let mut nds = Nds::new(firmware_bytes, bios7_bytes, bios9_bytes, rom_bytes, skip_bios);

  let sdl_context = sdl2::init().unwrap();

  let mut frontend = Frontend::new(&sdl_context);

  let mut frame_finished = false;

  loop {
    while !frame_finished {
      frame_finished = nds.step();
    }

    let ref mut bus = *nds.bus.borrow_mut();

    bus.gpu.frame_finished = false;
    frame_finished = false;

    // render stuff
    frontend.render(&mut bus.gpu);
    frontend.handle_events(bus);
    frontend.push_samples(bus.arm7.apu.audio_samples.drain(..).collect())
  }
}