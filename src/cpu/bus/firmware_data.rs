pub const FIRMWARE_CAPACITY: usize = 0x40000;
pub const FIRMWARE_MASK: usize = FIRMWARE_CAPACITY - 1;

pub const MAC_ADDRESS: [u8; 6] = [0x00, 0x09, 0xBF, 0x11, 0x22, 0x33];
pub const INITIAL_BB_VALUES: [u8; 0x69] = [
  0x03, 0x17, 0x40, 0x00, 0x1B, 0x6C, 0x48, 0x80, 0x38, 0x00, 0x35, 0x07, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC7, 0xBB, 0x01, 0x24, 0x7F,
  0x5A, 0x01, 0x3F, 0x01, 0x3F, 0x36, 0x1D, 0x00, 0x78, 0x35, 0x55, 0x12, 0x34, 0x1C, 0x00, 0x01,
  0x0E, 0x38, 0x03, 0x70, 0xC5, 0x2A, 0x0A, 0x08, 0x04, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFE,
  0xFE, 0xFE, 0xFE, 0xFC, 0xFC, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xF8, 0xF8, 0xF6, 0x00, 0x12, 0x14,
  0x12, 0x41, 0x23, 0x03, 0x04, 0x70, 0x35, 0x0E, 0x2C, 0x2C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x12, 0x28, 0x1C
];

pub const INITIAL_RF_VALUES: [u8; 41] = [
  0x31, 0x4C, 0x4F, 0x21, 0x00, 0x10, 0xB0, 0x08, 0xFA, 0x15, 0x26, 0xE6, 0xC1, 0x01, 0x0E, 0x50,
  0x05, 0x00, 0x6D, 0x12, 0x00, 0x00, 0x01, 0xFF, 0x0E, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x06,
  0x06, 0x00, 0x00, 0x00, 0x18, 0x00, 0x02, 0x00, 0x00
];

pub const BB_DATA1: [u8; 14] = [
  0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x16,
];

pub const BB_DATA2: [u8; 14] = [
  0x1C, 0x1C, 0x1C, 0x1D, 0x1D, 0x1D, 0x1E, 0x1E, 0x1E, 0x1E, 0x1F, 0x1E, 0x1F, 0x18,
];

pub const RF_DATA1: [u8; 14] = [
  0x4B, 0x4B, 0x4B, 0x4B, 0x4C, 0x4C, 0x4C, 0x4C, 0x4C, 0x4C, 0x4C, 0x4D, 0x4D, 0x4D,
];

pub const RF_DATA2: [u8; 14] = [
  0x6C, 0x71, 0x76, 0x5B, 0x40, 0x45, 0x4A, 0x2F, 0x34, 0x39, 0x3E, 0x03, 0x08, 0x14
];

pub const DEFAULT_UNUSED3: [u8; 6] = [
  0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00
];

#[derive(Copy, Clone)]
pub enum RFChipType {
  Type2 = 0x2,
  Type3 = 0x3
}

#[derive(Copy, Clone)]
pub enum WifiVersion {
  W006 = 6
}

#[derive(Copy, Clone)]
pub enum ConsoleType {
  DS = 0xFF,
  DSLite = 0x20
}

pub struct UserSettings {
  version: u16,
  nickname: String,
  birthday_month: u8,
  birthday_day: u8,
  favorite_color: u8,
  name_length: u16,
  message: String,
  message_length: u16,
  settings: u16,
  unused2: [u8; 4],
  // checksum: u16
}

impl UserSettings {
  pub fn new() -> Self {
    Self {
      version: 5,
      birthday_month: 5,
      birthday_day: 24,
      settings: 1 | (3 << 4), // english, max lighting level
      nickname: "NDS Plus".to_string(),
      name_length: "NDS Plus".len() as u16,
      // checksum: 0, // TODO: actually calculate this
      favorite_color: 2,
      unused2: [0xff; 4],
      message: "Hello!".to_string(),
      message_length: "Hello!".len() as u16,
    }
  }

  pub fn fill_buffer(&self, buffer: &mut [u8]) {
    let user_settings_base = 0x3fe00;

    unsafe { *(&mut buffer[user_settings_base] as *mut u8 as *mut u16) = self.version };
    buffer[user_settings_base + 0x2] = self.favorite_color;
    buffer[user_settings_base + 0x3] = self.birthday_month;
    buffer[user_settings_base + 0x4] = self.birthday_day;

    buffer[user_settings_base + 0x6..user_settings_base + 0x6 + self.nickname.len()].copy_from_slice(self.nickname.as_bytes().try_into().unwrap());

    unsafe { *(&mut buffer[user_settings_base + 0x1a] as *mut u8 as *mut u16) = self.name_length };

    buffer[user_settings_base + 0x1c..user_settings_base + 0x1c + self.message.len()].copy_from_slice(self.message.as_bytes().try_into().unwrap());
    unsafe { *(&mut buffer[user_settings_base + 0x50] as *mut u8 as *mut u16) = self.message_length };

    unsafe { *(&mut buffer[user_settings_base + 0x64] as *mut u8 as *mut u16) = self.settings };

    buffer[user_settings_base + 0x6c..user_settings_base + 0x6c + 4].copy_from_slice(&self.unused2[0..4]);
  }
}

pub struct FirmwareHeader {
  // arm9_gui_code_offset: u16,
  // arm7_wifi_code_offset: u16,
  // gui_wifi_code_checksum: u16,
  // boot_code_checksum: u16,
  // arm9_boot_code_rom_address: u16,
  // arm9_boot_code_ram_address: u16,
  // arm7_boot_code_ram_address: u16,
  // arm7_boot_code_rom_address: u16,
  // shift_amounts: u16,
  // data_gfx_rom_address: u16,
  // build_minute: u8,
  // build_hour: u8,
  // build_day: u8,
  // build_month: u8,
  // build_year: u8,
  user_settings_offset: u16,
  // data_gfx_checksom: u16,
  console_type: ConsoleType,
  identifier: [u8; 4],
  wifi_config_length: u16,
  wifi_version: WifiVersion,
  mac_address: [u8; 6],
  // enabled_channels: u16,
  rf_chip_type: RFChipType,
  rf_bits_per_entry: u8,
  rf_entries: u8,
  initial_values: [u16; 16],
  initial_bb_values: [u8; 0x69],
  initial_rf_values: [u8; 41],
  bb_indices_per_channel: u8,
  bb_index1: u8,
  bb_data1: [u8; 14],
  bb_index2: u8,
  bb_data2: [u8; 14],
  rf_index1: u8,
  rf_data1: [u8; 14],
  rf_index2: u8,
  rf_data2: [u8; 14],
  unused0: [u8; 46],
  wifi_board: u8,
  wifi_flash: u8,
  dsi_3ds: u8,
  unknown2: [u8; 2],
  unknown3: u8,
  unused3: [u8; 6],
  wifi_config_checksum: u8

}

impl FirmwareHeader {
  pub fn new() -> Self {
    Self {
      // arm7_boot_code_ram_address: 0,
      // arm7_boot_code_rom_address: 0,
      // gui_wifi_code_checksum: 0,
      // boot_code_checksum: 0,
      // arm9_boot_code_ram_address: 0,
      // arm7_wifi_code_offset: 0,
      // arm9_boot_code_rom_address: 0,
      // arm9_gui_code_offset: 0,
      // shift_amounts: 0,
      // data_gfx_rom_address: 0,
      console_type: ConsoleType::DSLite,
      identifier: "NDSP".as_bytes().try_into().unwrap(), // NDS Plus
      wifi_version: WifiVersion::W006,
      rf_chip_type: RFChipType::Type3,
      // build_day: 0,
      // build_hour: 0,
      // build_minute: 0,
      // build_month: 0,
      // build_year: 0,
      user_settings_offset: ((0x7FE00 & FIRMWARE_MASK) >> 3) as u16,
      // data_gfx_checksom: 0,
      wifi_config_length: 0x138,
      mac_address: MAC_ADDRESS,
      // enabled_channels: 0x3ffe,
      rf_bits_per_entry: 0x94,
      rf_entries: 0x29,
      initial_values: [0; 16],
      initial_bb_values: INITIAL_BB_VALUES,
      initial_rf_values: INITIAL_RF_VALUES,
      wifi_board: 0xff,
      wifi_flash: 0xff,
      dsi_3ds: 0xff,
      unknown2: [0xff; 2],
      unknown3: 0x2,
      bb_indices_per_channel: 2,
      bb_index1: 0x1e,
      bb_data1: BB_DATA1,
      bb_index2: 0x26,
      bb_data2: BB_DATA2,
      rf_index1: 0x1,
      rf_data1: RF_DATA1,
      rf_index2: 0x02,
      rf_data2: RF_DATA2,
      unused0: [0xff; 46],
      unused3: DEFAULT_UNUSED3,
      wifi_config_checksum: 0
    }
  }

  pub fn fill_buffer(&self, buffer: &mut [u8]) {
    // console_type, identifier, wifi_version, rf_chip_type
    // user_settings_offset, wifi_config_length, mac_address,
    // enabled_channels, rf_bits_per_entry, rf_entries, initial_values,
    // initial_bb_values, initial_rf_values, wifi_board, wifi_flash,
    // dsi_3ds, unknown2, unknown3, bb_indices_per_channel, bb_index...,
    // bb_data..... rf_index... rf_data...., unused0, unused3

    // firmware header
    unsafe { *(&mut buffer[0x8] as *mut u8 as *mut u32) = *(&self.identifier[0] as *const u8 as *const u32) };
    buffer[0x1d] = self.console_type as u8;
    unsafe { *(&mut buffer[0x20] as *mut u8 as *mut u16) = self.user_settings_offset };

    // wifi settings
    unsafe { *(&mut buffer[0x2c] as *mut u8 as *mut u16) = self.wifi_config_length };

    buffer[0x2f] = self.wifi_version as u8;

    buffer[0x36..0x3c].copy_from_slice(&self.mac_address[0..6]);
    buffer[0x3e..0x40].fill(0xff);

    buffer[0x40] = self.rf_chip_type as u8;
    buffer[0x41] = self.rf_bits_per_entry;
    buffer[0x42] = self.rf_entries;
    buffer[0x43] = 0x1;

    let mut firm_address = 0x44;

    for i in 0..self.initial_values.len() {
      unsafe { *(&mut buffer[firm_address] as *mut u8 as *mut u16) = self.initial_values[i] };
      firm_address += 2;
    }

    buffer[0x64..0x64+0x69].copy_from_slice(&self.initial_bb_values[0..0x69]);
    buffer[0xce..0xce+41].copy_from_slice(&self.initial_rf_values[0..41]);

    buffer[0xf7] = self.bb_indices_per_channel;
    buffer[0xf8] = self.bb_index1;
    buffer[0xf9..0xf9 + 14].copy_from_slice(&self.bb_data1[0..14]);
    buffer[0x107] = self.bb_index2;
    buffer[0x108..0x108 + 14].copy_from_slice(&self.bb_data2[0..14]);
    buffer[0x116] = self.rf_index1;
    buffer[0x117..0x117 + 14].copy_from_slice(&self.rf_data1[0..14]);
    buffer[0x125] = self.rf_index2;
    buffer[0x126..0x126 + 14].copy_from_slice(&self.rf_data2[0..14]);

    buffer[0x134..0x134 + 46].copy_from_slice(&self.unused0[0..46]);

    buffer[0x1fd] = self.wifi_board;
    buffer[0x1fe] = self.wifi_flash;
    buffer[0x1ff] = self.dsi_3ds;
  }
}

pub struct FirmwareData {
  pub header: FirmwareHeader,
  pub user_settings: UserSettings
}

impl FirmwareData {
  pub fn new(buffer: &mut [u8]) -> Self {
    let mut firmware_data = Self {
      header: FirmwareHeader::new(),
      user_settings: UserSettings::new()
    };
    let header = &mut firmware_data.header;
    let user_settings = &mut firmware_data.user_settings;

    header.initial_values[0] = 0x0002;
    header.initial_values[1] = 0x0017;
    header.initial_values[2] = 0x0026;
    header.initial_values[3] = 0x1818;
    header.initial_values[4] = 0x0048;
    header.initial_values[5] = 0x4840;
    header.initial_values[6] = 0x0058;
    header.initial_values[7] = 0x0042;
    header.initial_values[8] = 0x0146;
    header.initial_values[9] = 0x8064;
    header.initial_values[10] = 0xE6E6;
    header.initial_values[11] = 0x2443;
    header.initial_values[12] = 0x000E;
    header.initial_values[13] = 0x0001;
    header.initial_values[14] = 0x0001;
    header.initial_values[15] = 0x0402;

    buffer[0..0x1d].fill(0);

    buffer[0x2ff] = 0x80;

    header.fill_buffer(buffer);
    user_settings.fill_buffer(buffer);

    firmware_data
  }
}