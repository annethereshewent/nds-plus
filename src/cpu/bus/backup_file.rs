use std::{fs::{self, File}, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Default)]
pub struct BackupFile {
  pub buffer: Vec<u8>,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub file: Option<File>,
  pub has_written: bool,
  pub last_write: u128,
  pub is_desktop: bool,
  path: Option<PathBuf>
}

impl BackupFile {
  pub fn new(
    path: Option<PathBuf>,
    bytes: Option<Vec<u8>>,
    capacity: usize,
    is_desktop: bool
  ) -> Self {
    if path.is_some() {
      let path = path.unwrap();
      if !path.is_file() {
        let mut file = File::create(&path).unwrap();
        file.write_all(&vec![0xff; capacity]).unwrap();
      }

      let path_clone = path.clone();

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
        has_written: false,
        last_write: 0,
        path: Some(path_clone),
        is_desktop
      }
    } else if bytes.is_some() {
      let bytes = bytes.unwrap();

      let buffer = if bytes.len() == capacity {
        bytes
      } else {
        vec![0xff; capacity]
      };

      Self {
        file: None,
        buffer,
        has_written: false,
        last_write: 0,
        path,
        is_desktop
      }
    } else {
      let buffer = vec![0; capacity];
      Self {
        file: None,
        buffer,
        has_written: false,
        last_write: 0,
        path,
        is_desktop
      }
    }
  }

  pub fn reset(&mut self) -> Self {
    let mut file: Option<File> = None;
    if let Some(path) = &self.path {
      file = Some(fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap());

      self.flush();
    }


    Self {
      buffer: self.buffer.clone(),
      file,
      has_written: false,
      last_write: 0,
      path: self.path.clone(),
      is_desktop: self.is_desktop
    }
  }

  pub fn read(&self, address: usize) -> u8 {
    self.buffer[address]
  }

  pub fn write(&mut self, address: usize, value: u8) {
    self.buffer[address] = value;
  }

  pub fn flush(&mut self) {
    if self.file.is_some() {
      let mut file = self.file.as_ref().unwrap();
      file.seek(SeekFrom::Start(0)).unwrap();
      file.write_all(&self.buffer).unwrap();
    }
  }
}