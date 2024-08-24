use std::{fs::{self, File}, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};

pub struct BackupFile {
  pub buffer: Vec<u8>,
  file: Option<File>,
  pub has_written: bool
}

impl BackupFile {
  pub fn new(path: Option<PathBuf>, bytes: Option<Vec<u8>>, capacity: usize) -> Self {
    if path.is_some() {
      let path = path.unwrap();
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
        file: Some(file),
        buffer,
        has_written: false
      }
    } else if bytes.is_some() {
      let bytes = bytes.unwrap();

      let buffer = if bytes.len() > 0 {
        bytes
      } else {
        let mut buf = Vec::with_capacity(capacity);
        for _ in 0..capacity {
          buf.push(0xff);
        }

        buf
      };

      Self {
        file: None,
        buffer,
        has_written: false
      }
    } else {
      panic!("Neither bytes nor path provided!");
    }

  }

  pub fn read(&self, address: usize) -> u8 {
    self.buffer[address]
  }

  pub fn write(&mut self, address: usize, value: u8) {
    self.buffer[address] = value;

    self.has_written = true;

    if self.file.is_some() {
      self.file.as_ref().unwrap().seek(SeekFrom::Start(address as u64)).unwrap();

      self.file.as_ref().unwrap().write_all(&[value]).unwrap();
    }

  }

  pub fn flush(&mut self) {
    if self.file.is_some() {
      self.file.as_ref().unwrap().seek(SeekFrom::Start(0)).unwrap();
      self.file.as_ref().unwrap().write_all(&self.buffer).unwrap();
    }
  }
}