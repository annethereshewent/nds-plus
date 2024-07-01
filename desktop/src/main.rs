use std::{env, fs};

use ds_emulator::nds::Nds;

extern crate ds_emulator;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() != 2 {
    panic!("please specify a rom file.");
  }

  let bios7_file = "../bios7.bin";
  let bios9_file = "../bios9.bin";
  let firmware_file = "../firmware.bin";

  let bios7_bytes = fs::read(bios7_file).unwrap();
  let bios9_bytes = fs::read(bios9_file).unwrap();
  let rom_bytes = fs::read(&args[1]).unwrap();
  let firmware_bytes = fs::read(firmware_file).unwrap();

  let mut nds = Nds::new(firmware_bytes, bios7_bytes, bios9_bytes, rom_bytes);

  loop {
    nds.step();
  }
}