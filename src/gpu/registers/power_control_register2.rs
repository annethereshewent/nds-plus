bitflags! {
  pub struct PowerControlRegister2: u16 {
    const SOUND_SPEAKERS_ENABLE = 1;
    const WIFI_ENABLE = 1 << 1;
  }
}