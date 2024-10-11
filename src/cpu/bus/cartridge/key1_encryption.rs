// Initial Encryption Values
// Below formulas can be used only with a copy of the 1048h-byte key tables from NDS/DSi BIOS.
// The values can be found at: NDS.ARM7 ROM: 00000030h..00001077h
pub const KEY_TABLE_SIZE: usize = 0x1048 / 4;

#[derive(Default)]
pub struct Key1Encryption {
  internal_key_buf: Box<[u32]>,
  pub key_buf: Box<[u32]>,
  pub ready: bool
}

impl Key1Encryption {
  pub fn new(bios7_bytes: &[u8]) -> Self {
    let mut key1 = Self {
      internal_key_buf: vec![0; KEY_TABLE_SIZE].into_boxed_slice(),
      key_buf: vec![0; KEY_TABLE_SIZE].into_boxed_slice(),
      ready: false
    };

    key1.load_key_table(bios7_bytes);

    key1
  }

  fn load_key_table(&mut self, bios: &[u8]) {
    let mut buf_index = 0;
    for i in (0x30..=0x1077).step_by(4) {
      self.internal_key_buf[buf_index] = u32::from_le_bytes(bios[i..i+4].try_into().unwrap());
      buf_index += 1;
    }
  }

  pub fn init_keycode(&mut self, id: u32, level: u32, modulo: u32) {
    // see https://www.problemkaputt.de/gbatek.htm#dsencryptionbygamecodeidcodekey1
    self.ready = true;

    self.key_buf = self.internal_key_buf.clone();

    let mut key_code = [id, id / 2, id * 2];

    if level >= 1{
      self.apply_keycode(&mut key_code, modulo);
    }
    if level >= 2 {
      self.apply_keycode(&mut key_code, modulo);
    }

    key_code[1] <<= 1;
    key_code[2] >>= 1;

    if level >= 3 {
      self.apply_keycode(&mut key_code, modulo);
    }
  }
  /*
  encrypt_64bit(ptr) / decrypt_64bit(ptr)
  Y=[ptr+0]
  X=[ptr+4]
  FOR I=0 TO 0Fh (encrypt), or FOR I=11h TO 02h (decrypt)
    Z=[keybuf+I*4] XOR X
    X=[keybuf+048h+((Z SHR 24) AND FFh)*4]
    X=[keybuf+448h+((Z SHR 16) AND FFh)*4] + X
    X=[keybuf+848h+((Z SHR  8) AND FFh)*4] XOR X
    X=[keybuf+C48h+((Z SHR  0) AND FFh)*4] + X
    X=Y XOR X
    Y=Z
  NEXT I
  [ptr+0]=X XOR [keybuf+40h] (encrypt), or [ptr+0]=X XOR [keybuf+4h] (decrypt)
  [ptr+4]=Y XOR [keybuf+44h] (encrypt), or [ptr+4]=Y XOR [keybuf+0h] (decrypt)

  */
  fn encrypt_decrypt64bit(&mut self, ptr: &mut [u32], is_decryption: bool) {
    let mut y = ptr[0];
    let mut x = ptr[1];

    let mut encrypt_range = 0x0..=0xF as usize;
    let mut decrypt_range = (0x2..=0x11 as usize).rev();
    let range = if !is_decryption {
        &mut encrypt_range as &mut dyn Iterator<Item = _>
    } else {
        &mut decrypt_range
    };


    for i in range {
      let z = self.key_buf[i] ^ x;
      x = self.key_buf[(0x48 / 4 + ((z >> 24) & 0xff)) as usize];
      x = self.key_buf[(0x448 / 4 + ((z >> 16) & 0xff)) as usize] + x;
      x = self.key_buf[(0x848 / 4 + ((z >> 8) & 0xff)) as usize] ^ x;
      x = self.key_buf[(0xc48 / 4 + (z & 0xff)) as usize] + x;

      x = y ^ x;
      y = z;
    }

    if !is_decryption {
      ptr[0] = x ^ self.key_buf[0x40 / 4];
      ptr[1] = y ^ self.key_buf[0x44 / 4];
    } else {
      ptr[0] = x ^ self.key_buf[1];
      ptr[1] = y ^ self.key_buf[0];
    }
  }

  pub fn decrypt_64bit(&mut self, ptr: &mut [u32]) {
    self.encrypt_decrypt64bit(ptr, true);
  }

  pub fn encrypt_64bit(&mut self, ptr: &mut [u32]) {
    self.encrypt_decrypt64bit(ptr, false);
  }
  /*
    apply_keycode(modulo)
      encrypt_64bit(keycode+4)
      encrypt_64bit(keycode+0)
      [scratch]=0000000000000000h   ;S=0 (64bit)
      FOR I=0 TO 44h STEP 4         ;xor with reversed byte-order (bswap)
        [keybuf+I]=[keybuf+I] XOR bswap_32bit([keycode+(I MOD modulo)])
      NEXT I
      FOR I=0 TO 1040h STEP 8
        encrypt_64bit(scratch)      ;encrypt S (64bit) by keybuf
        [keybuf+I+0]=[scratch+4]    ;write S to keybuf (first upper 32bit)
        [keybuf+I+4]=[scratch+0]    ;write S to keybuf (then lower 32bit)
      NEXT I
   */
  fn apply_keycode(&mut self, key_code: &mut [u32], modulo: u32) {
    self.encrypt_64bit(&mut key_code[1..3]);
    self.encrypt_64bit(&mut key_code[0..2]);

    let mut scratch = [0, 0];

    for i in 0..=0x44 / 4 {
      self.key_buf[i] ^= key_code[i % (modulo as usize)].swap_bytes();
    }
    for i in (0..=0x1040 / 4).step_by(2) {
      self.encrypt_64bit(&mut scratch);
      self.key_buf[i] = scratch[1];
      self.key_buf[i + 1] = scratch[0];
    }
  }
}