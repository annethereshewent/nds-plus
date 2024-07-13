use std::{collections::HashMap, env, fs};

use ds_emulator::{cpu::registers::key_input_register::KeyInputRegister, gpu::{SCREEN_HEIGHT, SCREEN_WIDTH}, nds::Nds};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect};

extern crate ds_emulator;

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
  let video_subsystem = sdl_context.video().unwrap();

  let window = video_subsystem
    .window("DS Emulator", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3 * 2) as u32)
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  canvas.set_scale(3.0, 3.0).unwrap();

  let mut event_pump = sdl_context.event_pump().unwrap();

  let creator = canvas.texture_creator();
  let mut texture_a = creator
    .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
    .unwrap();

  let mut texture_b = creator
    .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
    .unwrap();

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

  let mut frame_finished = false;

  loop {
    while !frame_finished {
      frame_finished = nds.step();
    }

    let ref mut bus = *nds.bus.borrow_mut();

    bus.gpu.frame_finished = false;
    frame_finished = false;

    // render stuff
    texture_a.update(None, &bus.gpu.engine_a.pixels, SCREEN_WIDTH as usize * 3).unwrap();
    texture_b.update(None, &bus.gpu.engine_b.pixels, SCREEN_WIDTH as usize * 3).unwrap();

    let screen_a = Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let screen_b = Rect::new(0, SCREEN_HEIGHT as i32, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);

    canvas.copy(&texture_a, None, screen_a).unwrap();
    canvas.copy(&texture_b, None, screen_b).unwrap();

    canvas.present();

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => std::process::exit(0),
        Event::KeyDown { keycode, .. } => {
          if let Some(button) = key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            bus.key_input_register.set(*button, false);
          } else if keycode.unwrap() == Keycode::G {
            nds.arm9_cpu.debug_on = !nds.arm9_cpu.debug_on;
            nds.arm7_cpu.debug_on = !nds.arm7_cpu.debug_on;
          } else if keycode.unwrap() == Keycode::F {
            bus.gpu.engine_a.debug_on = !bus.gpu.engine_a.debug_on;
            bus.gpu.engine_b.debug_on = !bus.gpu.engine_b.debug_on;
          }
        }
        Event::KeyUp { keycode, .. } => {
          if let Some(button) = key_map.get(&keycode.unwrap_or(Keycode::Return)) {
            bus.key_input_register.set(*button, true);
          }
        }
        _ => ()
      }
    }
  }
}