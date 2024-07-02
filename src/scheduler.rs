use std::cmp::Reverse;

use priority_queue::PriorityQueue;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum EventType {
  HBLANK,
  NEXT_LINE
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

  pub fn update_cycles(&mut self, cycles: usize) {
    self.cycles = cycles;
  }

  pub fn get_next_event(&mut self) -> Option<(EventType, usize)> {
    if let Some((event_type, Reverse(cycles))) = self.queue.pop() {
      Some((event_type, cycles))
    } else {
      None
    }
  }
}

