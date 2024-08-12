use super::{color::Color, engine_2d::{Engine2d, ObjectPixel}, registers::{alpha_blend_register::AlphaBlendRegister, bg_control_register::BgControlRegister, brightness_register::BrightnessRegister, color_effects_register::ColorEffectsRegister, display_control_register::DisplayControlRegister, master_brightness_register::MasterBrightnessRegister, window_horizontal_register::WindowHorizontalRegister, window_in_register::WindowInRegister, window_out_register::WindowOutRegister, window_vertical_register::WindowVerticalRegister}, BgProps, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct RenderingData {
  pub dispcnt: DisplayControlRegister,
  pub bgcnt: [BgControlRegister; 4],
  pub oam: [u8; 0x400],
  pub pixels: [u8; 3 * (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
  pub winin: WindowInRegister,
  pub winout: WindowOutRegister,
  pub winh: [WindowHorizontalRegister; 2],
  pub winv: [WindowVerticalRegister; 2],
  pub bldcnt: ColorEffectsRegister,
  pub bldalpha: AlphaBlendRegister,
  pub bldy: BrightnessRegister,
  pub bgxofs: [u16; 4],
  pub bgyofs: [u16; 4],
  pub bg_props: [BgProps; 2],
  pub bg_lines: [[Option<Color>; SCREEN_WIDTH as usize]; 4],
  pub obj_lines: [ObjectPixel; SCREEN_WIDTH as usize],
  pub master_brightness: MasterBrightnessRegister,
  pub palette_ram: [u8; 0x400],
}

impl RenderingData {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      bgcnt: [BgControlRegister::from_bits_retain(0); 4],
      oam: [0; 0x400],
      pixels: [0; 3 * (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
      bgxofs: [0; 4],
      bgyofs: [0; 4],
      bg_props: [BgProps::new(); 2],
      winh: [WindowHorizontalRegister::new(); 2],
      winv: [WindowVerticalRegister::new(); 2],
      winin: WindowInRegister::from_bits_retain(0),
      winout: WindowOutRegister::from_bits_retain(0),
      bldcnt: ColorEffectsRegister::new(),
      bldalpha: AlphaBlendRegister::new(),
      bldy: BrightnessRegister::new(),
      bg_lines: [[None; SCREEN_WIDTH as usize]; 4],
      master_brightness: MasterBrightnessRegister::new(),
      palette_ram: [0; 0x400],
      obj_lines: [ObjectPixel::new(); SCREEN_WIDTH as usize],
    }
  }
}