use std::{cmp::Reverse, collections::HashMap};

use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum EventType {
  HBlank,
  HDraw,
  Timer7(usize),
  Timer9(usize),
  BlockFinished(bool),
  WordTransfer(bool),
  StepAudio(usize),
  ResetAudio(usize),
  GenerateSample,
  CheckGeometryFifo
}

#[derive(Serialize, Deserialize)]
pub struct Scheduler {
  pub cycles: usize,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub queue: PriorityQueue<EventType, Reverse<usize>>,
  pub queue_serialized: HashMap<EventType, usize>
}

impl Scheduler {
  pub fn new() -> Self {
    Self {
      cycles: 0,
      queue: PriorityQueue::new(),
      queue_serialized: HashMap::new()
    }
  }

  pub fn schedule(&mut self, event_type: EventType, time: usize) {
    self.queue.push(event_type, Reverse(self.cycles + time));
  }

  pub fn remove(&mut self, event_type: EventType) {
    self.queue.remove(&event_type);
  }

  pub fn update_cycles(&mut self, cycles: usize) {
    self.cycles = cycles;
  }

  pub fn get_next_event(&mut self) -> Option<(EventType, usize)> {
    let (_, Reverse(cycles)) = self.queue.peek().unwrap();

    if self.cycles >= *cycles {
      let cycles_left = self.cycles - *cycles;
      let (event_type, _) = self.queue.pop().unwrap();
      return Some((event_type, cycles_left));
    }

    None
  }

  pub fn rebase_cycles(&mut self) -> usize {
    let to_subtract = self.cycles;

    self.cycles = 0;

    let mut vec: Vec<(EventType, usize)> = Vec::new();

    while let Some((event_type, Reverse(cycles)))= self.queue.pop() {
      let new_cycles = cycles - to_subtract;

      vec.push((event_type, new_cycles));
    }

    for (event_type, cycles) in vec {
      self.queue.push(event_type, Reverse(cycles));
    }

    to_subtract
  }

  pub fn create_save_state(&mut self) {
    for (event_type, Reverse(cycles)) in self.queue.iter() {
      self.queue_serialized.insert(*event_type, *cycles);
    }
  }

  pub fn load_save_state(&mut self) {
    for (event_type, cycles) in self.queue_serialized.iter() {
      self.queue.push(*event_type, Reverse(*cycles));
    }
  }

  pub fn get_cycles_to_next_event(&mut self) -> usize {
    if let Some((_, Reverse(cycles))) = self.queue.peek() {
      *cycles
    } else {
      0
    }
  }
}

