use ds_emulator::nds::Nds;

extern crate ds_emulator;

fn main() {
  let mut nds = Nds::new();

  loop {
    nds.step();
  }
}