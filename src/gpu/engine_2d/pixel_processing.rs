use std::cmp;

use crate::gpu::{
  registers::{
    color_effects_register::ColorEffect,
    display_control_register::DisplayControlRegisterFlags,
    window_in_register::WindowInRegister,
    window_out_register::WindowOutRegister
  },
  SCREEN_WIDTH
};

use super::{Color, Engine2d};

enum WindowType {
  Zero = 0,
  One = 1,
  Obj = 2,
  Out = 3,
  None = 4
}

#[derive(Copy, Clone, Debug)]
struct Layer {
  index: usize,
  priority: usize
}

impl Layer {
  pub fn new(index: usize, priority: usize) -> Self {
    Self {
      index,
      priority
    }
  }
}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn finalize_scanline(&mut self, y: u16) {
    let mut sorted: Vec<usize> = Vec::new();

    for i in 0..=3 {
      if self.bg_mode_enabled(i) {
        sorted.push(i);
      }
    }

    sorted.sort_by_key(|key| (self.bgcnt[*key].bg_priority(), *key));

    let mut occupied = [false; SCREEN_WIDTH as usize];

    if self.dispcnt.windows_enabled() {
      if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW0) {
        let mut sorted_window_layers: Vec<usize> = Vec::new();
        if y >= self.winv[0].y1 && y < self.winv[0].y2 {

          for bg in &sorted {
            if self.winin.window0_bg_enable() >> bg & 0b1 == 1 {
              sorted_window_layers.push(*bg);
            }
          }

          for x in self.winh[0].x1..self.winh[0].x2 {
            if !occupied[x as usize] {
             self.finalize_pixel(x, y, &sorted_window_layers, WindowType::Zero);
             occupied[x as usize] = true;
            }
          }
        }
      }

      if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW1) {
        let mut sorted_window_layers: Vec<usize> = Vec::new();
        if y >= self.winv[1].y1 && y < self.winv[1].y2 {
          for bg in &sorted {
            if self.winin.window1_bg_enable() >> bg & 0b1 == 1 {
              sorted_window_layers.push(*bg);
            }
          }

          for x in self.winh[1].x1..self.winh[1].x2 {
            if !occupied[x as usize] {
              self.finalize_pixel(x, y, &sorted_window_layers, WindowType::One);
              occupied[x as usize] = true;
            }
          }
        }
      }


      // finally do outside window layers
      let mut outside_layers: Vec<usize> = Vec::new();
      for bg in &sorted {
        if self.winout.outside_window_background_enable_bits() >> bg & 0b1 == 1 {
          outside_layers.push(*bg);
        }
      }

      if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ_WINDOW) {
        for x in 0..SCREEN_WIDTH {
          if !occupied[x as usize] {
            if self.obj_lines[x as usize].is_window {
              self.finalize_pixel(x, y, &outside_layers, WindowType::Obj);
            } else {
              self.finalize_pixel(x, y, &outside_layers, WindowType::Out);
            }
            occupied[x as usize] = true;
          }
        }
      }

      // lastly do any remaining outside window pixels
      for x in 0..SCREEN_WIDTH {
        if !occupied[x as usize] {
          self.finalize_pixel(x, y, &outside_layers, WindowType::Out);
          occupied[x as usize] = true;
        }
      }
    } else {
      // render like normal by priority
      for x in 0..SCREEN_WIDTH {
        if !occupied[x as usize] {
          self.finalize_pixel(x, y, &sorted, WindowType::None);
          occupied[x as usize] = true;
        }
      }
    }
  }

  fn display_window_obj(&self, window_type: &WindowType) -> bool {
    match window_type {
      WindowType::Zero => {
        self.winin.contains(WindowInRegister::Window0ObjEnable)
      }
      WindowType::One => {
        self.winin.contains(WindowInRegister::Window1ObjEnable)
      }
      WindowType::Obj => {
        self.winout.contains(WindowOutRegister::ObjWindowObjEnable)
      }
      WindowType::Out => {
        self.winout.contains(WindowOutRegister::OutsideWindowObjEnable)
      }
      WindowType::None => true
    }
  }

  fn finalize_pixel(&mut self, x: u16, y: u16, sorted_layers: &Vec<usize>, window_type: WindowType) {
    let mut bottom_layer: Option<Layer> = None;
    let mut top_layer: Option<Layer> = None;

    for bg in sorted_layers {
      if self.bg_lines[*bg][x as usize].is_some() {
        if top_layer.is_none() {
          top_layer = Some(Layer::new(*bg, self.bgcnt[*bg].bg_priority() as usize));
        } else {
          bottom_layer = Some(Layer::new(*bg, self.bgcnt[*bg].bg_priority() as usize));
          break;
        }
      }
    }

    let obj_layer = Layer::new(4, self.obj_lines[x as usize].priority as usize);

    if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ) && self.display_window_obj(&window_type) {
      if top_layer.is_none() || obj_layer.priority <= top_layer.unwrap().priority {
        bottom_layer = top_layer;
        top_layer = Some(obj_layer);
      } else if bottom_layer.is_none() || obj_layer.priority <= bottom_layer.unwrap().priority {
        bottom_layer = Some(obj_layer);
      }
    }

    let (top_layer_color, top_layer) = if let Some(layer) = top_layer {
      if layer.index < 4 {
        if self.bg_lines[layer.index][x as usize].is_some() {
          (self.bg_lines[layer.index][x as usize], Some(layer))
        } else if let Some(layer) = bottom_layer {
          if layer.index < 4 {
            (self.bg_lines[layer.index][x as usize], Some(layer))
          } else {
            (self.obj_lines[x as usize].color, Some(layer))
          }
        } else {
          (None, None)
        }
      } else {
        (self.obj_lines[x as usize].color, Some(layer))
      }
    } else {
      (None, None)
    };

    let default_color = Color::to_rgb24((self.palette_ram[0] as u16) | (self.palette_ram[1] as u16) << 8);

    if let Some(mut top_layer_color) = top_layer_color {
      // this is safe to do, as we've verified the top layer and color above
      let top_layer = top_layer.unwrap();
      // do further processing if needed

      if top_layer.index == 4 {
        if self.obj_lines[x as usize].is_transparent && bottom_layer.is_some() && self.bldcnt.bg_second_pixels[bottom_layer.unwrap().index] {
          let bottom_layer = bottom_layer.unwrap();

          if let Some(color2) = self.bg_lines[bottom_layer.index][x as usize] {
            top_layer_color = self.blend_colors(top_layer_color, color2, self.bldalpha.eva as u16, self.bldalpha.evb as u16);
          }
        }
      } else if self.bldcnt.bg_first_pixels[top_layer.index] && self.should_apply_effects(&window_type) {
        top_layer_color = self.process_pixel(x as usize, top_layer_color, bottom_layer);
      }

      // lastly apply master brightness
      top_layer_color = self.master_brightness.apply_effect(top_layer_color);

      self.set_pixel(x as usize, y as usize, top_layer_color.convert());
    } else {
      self.set_pixel(x as usize, y as usize, default_color);
    }

  }

  fn blend_colors(&self, color: Color, color2: Color, eva: u16, evb: u16) -> Color {
    let r = cmp::min(31, (color.r as u16 * eva + color2.r as u16 * evb) >> 4) as u8;
    let g = cmp::min(31, (color.g as u16 * eva + color2.g as u16 * evb) >> 4) as u8;
    let b = cmp::min(31, (color.b as u16 * eva + color2.b as u16 * evb) >> 4) as u8;

    Color {
      r,
      g,
      b,
      alpha: None
    }
  }

  fn blend_colors3d(&self, color: Color, color2: Color, eva: u16, evb: u16) -> Color {
    let r = cmp::min(31, (color.r as u16 * eva + color2.r as u16 * evb) >> 5) as u8;
    let g = cmp::min(31, (color.g as u16 * eva + color2.g as u16 * evb) >> 5) as u8;
    let b = cmp::min(31, (color.b as u16 * eva + color2.b as u16 * evb) >> 5) as u8;

    Color {
      r,
      g,
      b,
      alpha: None
    }
  }

  fn process_pixel(&mut self, x: usize, color: Color, bottom_layer: Option<Layer>) -> Color {
    match self.bldcnt.color_effect {
      ColorEffect::AlphaBlending => {
        let layer = if self.is_bottom_layer_blended(bottom_layer) {
          bottom_layer
        } else {
          None
        };

        if let Some(blend_layer) = layer {
          if blend_layer.index != 4 {
            if let Some(color2) = self.bg_lines[blend_layer.index][x] {
              if let Some(alpha) = color.alpha {
                let eva = alpha + 1;
                let evb = 32 - eva;
                self.blend_colors3d(color, color2, eva as u16, evb as u16)
              } else {
                self.blend_colors(color, color2, self.bldalpha.eva as u16, self.bldalpha.evb as u16)
              }
            } else {
              color
            }
          } else {
            if let Some(color2) = self.obj_lines[x].color {
              self.blend_colors(color, color2, self.bldalpha.eva as u16, self.bldalpha.evb as u16)
            } else {
              color
            }
          }

        } else {
          color
        }
      }
      ColorEffect::Brighten => {
          let white = Color {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            alpha: None
          };
          self.blend_colors(color, white, (16 - self.bldy.evy) as u16, self.bldy.evy as u16)

      }
      ColorEffect::Darken => {
        let black = Color {
          r: 0,
          g: 0,
          b: 0,
          alpha: None
        };

        self.blend_colors(color, black, (16 - self.bldy.evy) as u16, self.bldy.evy as u16)
      }
      ColorEffect::None => {
        color
      }
    }
  }

  fn is_bottom_layer_blended(&self, bottom_layer: Option<Layer>) -> bool {
    // (bottom_layer < 4 && bottom_layer >= 0 && self.bldcnt.bg_second_pixels[bottom_layer as usize]) || (bottom_layer == 4 && self.bldcnt.obj_second_pixel)

    if let Some(bottom_layer) = bottom_layer {
      (bottom_layer.index < 4 && self.bldcnt.bg_second_pixels[bottom_layer.index]) || (bottom_layer.index == 4 && self.bldcnt.obj_second_pixel)
    } else {
      false
    }
  }

  fn should_apply_effects(&self, window_type: &WindowType) -> bool {
    match window_type {
      WindowType::Zero => {
        self.winin.contains(WindowInRegister::Window0ColorEffect)
      }
      WindowType::One => {
        self.winin.contains(WindowInRegister::Window1ColorEffect)
      }
      WindowType::Obj => {
        self.winout.contains(WindowOutRegister::ObjWIndowColorEffect)
      }
      WindowType::Out => {
        self.winout.contains(WindowOutRegister::OutsideWindowColorEffect)
      }
      WindowType::None => true
    }
  }
}
