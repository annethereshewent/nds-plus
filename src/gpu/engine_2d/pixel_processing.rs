use std::cmp;

use crate::gpu::{
  engine_3d::Engine3d, registers::{
    color_effects_register::ColorEffect,
    display_control_register::DisplayControlRegisterFlags,
    window_in_register::WindowInRegister,
    window_out_register::WindowOutRegister
  }, rendering_data::RenderingData, SCREEN_WIDTH
};

use super::{renderer2d::Renderer2d, Color};

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

impl Renderer2d {
  pub fn finalize_scanline(y: u16, data: &mut RenderingData) {
    let mut sorted: Vec<usize> = Vec::new();

    for i in 0..=3 {
      if Self::bg_mode_enabled(i, data) {
        sorted.push(i);
      }
    }

    sorted.sort_by_key(|key| (data.bgcnt[*key].bg_priority(), *key));

    let mut occupied = [false; SCREEN_WIDTH as usize];

    if data.dispcnt.windows_enabled() {
      if data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW0) {
        let mut sorted_window_layers: Vec<usize> = Vec::new();
        if y >= data.winv[0].y1 && y < data.winv[0].y2 {

          for bg in &sorted {
            if data.winin.window0_bg_enable() >> bg & 0b1 == 1 {
              sorted_window_layers.push(*bg);
            }
          }

          for x in data.winh[0].x1..data.winh[0].x2 {
            if !occupied[x as usize] {
             Self::finalize_pixel(x, y, &sorted_window_layers, WindowType::Zero, data);
             occupied[x as usize] = true;
            }
          }
        }
      }

      if data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW1) {
        let mut sorted_window_layers: Vec<usize> = Vec::new();
        if y >= data.winv[1].y1 && y < data.winv[1].y2 {
          for bg in &sorted {
            if data.winin.window1_bg_enable() >> bg & 0b1 == 1 {
              sorted_window_layers.push(*bg);
            }
          }

          for x in data.winh[1].x1..data.winh[1].x2 {
            if !occupied[x as usize] {
              Self::finalize_pixel(x, y, &sorted_window_layers, WindowType::One, data);
              occupied[x as usize] = true;
            }
          }
        }
      }


      // finally do outside window layers
      let mut outside_layers: Vec<usize> = Vec::new();
      for bg in &sorted {
        if data.winout.outside_window_background_enable_bits() >> bg & 0b1 == 1 {
          outside_layers.push(*bg);
        }
      }

      if data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ_WINDOW) {
        for x in 0..SCREEN_WIDTH {
          if !occupied[x as usize] {
            if data.obj_lines[x as usize].is_window {
              Self::finalize_pixel(x, y, &outside_layers, WindowType::Obj, data);
            } else {
              Self::finalize_pixel(x, y, &outside_layers, WindowType::Out, data);
            }
            occupied[x as usize] = true;
          }
        }
      }

      // lastly do any remaining outside window pixels
      for x in 0..SCREEN_WIDTH {
        if !occupied[x as usize] {
          Self::finalize_pixel(x, y, &outside_layers, WindowType::Out, data);
          occupied[x as usize] = true;
        }
      }
    } else {
      // render like normal by priority
      for x in 0..SCREEN_WIDTH {
        if !occupied[x as usize] {
          Self::finalize_pixel(x, y, &sorted, WindowType::None, data);
          occupied[x as usize] = true;
        }
      }
    }
  }

  fn display_window_obj(window_type: &WindowType, data: &RenderingData) -> bool {
    match window_type {
      WindowType::Zero => {
        data.winin.contains(WindowInRegister::Window0ObjEnable)
      }
      WindowType::One => {
        data.winin.contains(WindowInRegister::Window1ObjEnable)
      }
      WindowType::Obj => {
        data.winout.contains(WindowOutRegister::ObjWindowObjEnable)
      }
      WindowType::Out => {
        data.winout.contains(WindowOutRegister::OutsideWindowObjEnable)
      }
      WindowType::None => true
    }
  }

  fn finalize_pixel(x: u16, y: u16, sorted_layers: &Vec<usize>, window_type: WindowType, data: &mut RenderingData) {
    let mut bottom_layer: Option<Layer> = None;
    let mut top_layer: Option<Layer> = None;

    for bg in sorted_layers {
      if data.bg_lines[*bg][x as usize].is_some() {
        if top_layer.is_none() {
          top_layer = Some(Layer::new(*bg, data.bgcnt[*bg].bg_priority() as usize));
        } else {
          bottom_layer = Some(Layer::new(*bg, data.bgcnt[*bg].bg_priority() as usize));
          break;
        }
      }
    }

    let obj_layer = Layer::new(4, data.obj_lines[x as usize].priority as usize);

    if data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ) && Self::display_window_obj(&window_type, data) {
      if top_layer.is_none() || obj_layer.priority <= top_layer.unwrap().priority {
        bottom_layer = top_layer;
        top_layer = Some(obj_layer);
      } else if bottom_layer.is_none() || obj_layer.priority <= bottom_layer.unwrap().priority {
        bottom_layer = Some(obj_layer);
      }
    }

    let (top_layer_color, top_layer) = if let Some(layer) = top_layer {
      if layer.index < 4 {
        if data.bg_lines[layer.index][x as usize].is_some() {
          (data.bg_lines[layer.index][x as usize], Some(layer))
        } else if let Some(layer) = bottom_layer {
          if layer.index < 4 {
            (data.bg_lines[layer.index][x as usize], Some(layer))
          } else {
            (data.obj_lines[x as usize].color, Some(layer))
          }
        } else {
          (None, None)
        }
      } else {
        (data.obj_lines[x as usize].color, Some(layer))
      }
    } else {
      (None, None)
    };

    let default_color = Color::to_rgb24((data.palette_ram[0] as u16) | (data.palette_ram[1] as u16) << 8);

    if let Some(mut top_layer_color) = top_layer_color {
      // this is safe to do, as we've verified the top layer and color above
      let top_layer = top_layer.unwrap();
      // do further processing if needed

      if top_layer.index == 4 {
        if data.obj_lines[x as usize].is_transparent && bottom_layer.is_some() && data.bldcnt.bg_second_pixels[bottom_layer.unwrap().index] {
          let bottom_layer = bottom_layer.unwrap();

          if let Some(color2) = data.bg_lines[bottom_layer.index][x as usize] {
            top_layer_color = Self::blend_colors(top_layer_color, color2, data.bldalpha.eva as u16, data.bldalpha.evb as u16);
          }
        }
      } else if data.bldcnt.bg_first_pixels[top_layer.index] && Self::should_apply_effects(&window_type, data) {
        top_layer_color = Self::process_pixel(x as usize, top_layer_color, bottom_layer, data);
      }

      // lastly apply master brightness
      top_layer_color = data.master_brightness.apply_effect(top_layer_color);

      data.pixel_alphas[x as usize] = true;
      Self::set_pixel(x as usize, y as usize, top_layer_color.convert(), data);
    } else {
      data.pixel_alphas[x as  usize] = false;
      Self::set_pixel(x as usize, y as usize, default_color, data);
    }

  }

  fn blend_colors(color: Color, color2: Color, eva: u16, evb: u16) -> Color {
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

  fn process_pixel(x: usize, color: Color, bottom_layer: Option<Layer>, data: &RenderingData) -> Color {
    match data.bldcnt.color_effect {
      ColorEffect::AlphaBlending => {
        let layer = if Self::is_bottom_layer_blended(bottom_layer, data) {
          bottom_layer
        } else {
          None
        };

        if let Some(blend_layer) = layer {
          if blend_layer.index != 4 {
            if let Some(color2) = data.bg_lines[blend_layer.index][x] {
              if let Some(alpha) = color.alpha {
                let eva = alpha + 1;
                let evb = 32 - eva;
                Engine3d::blend_colors3d(color, color2, eva as u16, evb as u16)
              } else {
                Self::blend_colors(color, color2, data.bldalpha.eva as u16, data.bldalpha.evb as u16)
              }
            } else {
              color
            }
          } else {
            if let Some(color2) = data.obj_lines[x].color {
              Self::blend_colors(color, color2, data.bldalpha.eva as u16, data.bldalpha.evb as u16)
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
          Self::blend_colors(color, white, (16 - data.bldy.evy) as u16, data.bldy.evy as u16)

      }
      ColorEffect::Darken => {
        let black = Color {
          r: 0,
          g: 0,
          b: 0,
          alpha: None
        };

        Self::blend_colors(color, black, (16 - data.bldy.evy) as u16, data.bldy.evy as u16)
      }
      ColorEffect::None => {
        color
      }
    }
  }

  fn is_bottom_layer_blended(bottom_layer: Option<Layer>, data: &RenderingData) -> bool {
    // (bottom_layer < 4 && bottom_layer >= 0 && self.bldcnt.bg_second_pixels[bottom_layer as usize]) || (bottom_layer == 4 && self.bldcnt.obj_second_pixel)

    if let Some(bottom_layer) = bottom_layer {
      (bottom_layer.index < 4 && data.bldcnt.bg_second_pixels[bottom_layer.index]) || (bottom_layer.index == 4 && data.bldcnt.obj_second_pixel)
    } else {
      false
    }
  }

  fn should_apply_effects(window_type: &WindowType, data: &RenderingData) -> bool {
    match window_type {
      WindowType::Zero => {
        data.winin.contains(WindowInRegister::Window0ColorEffect)
      }
      WindowType::One => {
        data.winin.contains(WindowInRegister::Window1ColorEffect)
      }
      WindowType::Obj => {
        data.winout.contains(WindowOutRegister::ObjWIndowColorEffect)
      }
      WindowType::Out => {
        data.winout.contains(WindowOutRegister::OutsideWindowColorEffect)
      }
      WindowType::None => true
    }
  }
}
