use std::cmp::Reverse;

use priority_queue::PriorityQueue;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum EventType {
  HBlank,
  HDraw,
  DMA7(usize),
  DMA9(usize),
  Timer7(usize),
  Timer9(usize),
  BlockFinished(bool),
  WordTransfer(bool),
  StepAudio(usize),
  ResetAudio(usize),
  GenerateSample,
  CheckGeometryFifo
}

pub struct Scheduler {
  pub cycles: usize,
  pub queue: PriorityQueue<EventType, Reverse<usize>>
}

impl Scheduler {
  pub fn new() -> Self {
    Self {
      cycles: 0,
      queue: PriorityQueue::new()
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

  pub fn get_cycles_to_next_event(&mut self) -> usize {
    if let Some((_, Reverse(cycles))) = self.queue.peek() {
      *cycles
    } else {
      0
    }
  }
}

