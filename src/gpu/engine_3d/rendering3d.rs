use super::Engine3d;

impl Engine3d {
  pub fn start_rendering(&mut self) {
    if self.polygons_ready {




      self.polygons_ready = false;
      self.gxstat.geometry_engine_busy = false;
    }
  }
}