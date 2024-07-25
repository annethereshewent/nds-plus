use std::{fs::{self, File}, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};

pub struct BackupFile {
  buffer: Vec<u8>,
  file: File
}

impl BackupFile {
  pub fn new(path: PathBuf, capacity: usize) -> Self {
    if !path.is_file() {
      let mut file = File::create(&path).unwrap();
      file.write_all(&vec![0xff; capacity]).unwrap();
    }

    let mut file = fs::OpenOptions::new()
      .read(true)
      .write(true)
      .open(path)
      .unwrap();

    let mut buffer = Vec::with_capacity(capacity);

    file.read_to_end(&mut buffer).unwrap();

    Self {
      file,
      buffer
    }
  }

  pub fn read(&self, address: usize) -> u8 {
    self.buffer[address]
  }

  pub fn write(&mut self, address: usize, value: u8) {
    self.buffer[address] = value;

    self.file.seek(SeekFrom::Start(address as u64)).unwrap();

    self.file.write_all(&[value]).unwrap();
  }

  pub fn flush(&mut self) {
    self.file.seek(SeekFrom::Start(0)).unwrap();
    self.file.write_all(&self.buffer).unwrap();
  }
}